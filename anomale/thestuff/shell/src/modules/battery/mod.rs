use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub fn build(charge_color: String) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("battery-module");
    
    let label = Label::new(Some("BAT[---]"));
    label.set_use_markup(true);
    container.append(&label);
    
    let label_clone = label.clone();
    let container_clone = container.clone();
    
    let charge_color_rc = Arc::new(charge_color);
    
    // Find the primary battery
    // Typically /sys/class/power_supply/BAT0 or BAT1, etc.
    // We'll search for one when the module is built
    let battery_path = Arc::new(Mutex::new(find_battery_path()));
    
    gtk4::glib::timeout_add_seconds_local(1, move || {
        let mut bpath = battery_path.lock().unwrap();
        
        // If no battery was found, maybe try finding it again occasionally? 
        // For now, assume if it's N/A, it stays N/A (like a desktop).
        if bpath.is_none() {
             *bpath = find_battery_path();
        }
        
        if let Some(path) = bpath.as_ref() {
            let capacity_path = path.join("capacity");
            let status_path = path.join("status");
            
            let mut capacity = 0;
            let mut status = String::from("Unknown");
            
            if let Ok(cap_str) = fs::read_to_string(&capacity_path) {
                if let Ok(cap) = cap_str.trim().parse::<u32>() {
                    capacity = cap;
                }
            }
            
            if let Ok(stat_str) = fs::read_to_string(&status_path) {
                status = stat_str.trim().to_string();
            }
            
            let display_text = if capacity >= 100 || status.eq_ignore_ascii_case("full") {
                "MAX".to_string()
            } else {
                format!("{:02}%", capacity)
            };
            
            if status.eq_ignore_ascii_case("charging") {
                // Apply charge color
                let markup_text = format!("BAT[<span color=\"{}\">{}</span>]", charge_color_rc, display_text);
                label_clone.set_markup(&markup_text);
                container_clone.add_css_class("charging");
            } else {
                // Normal
                label_clone.set_text(&format!("BAT[{}]", display_text));
                container_clone.remove_css_class("charging");
            }
        } else {
            label_clone.set_text("BAT[N/A]");
            container_clone.remove_css_class("charging");
        }
        
        gtk4::glib::ControlFlow::Continue
    });

    container
}

fn find_battery_path() -> Option<PathBuf> {
    let power_supply_dir = Path::new("/sys/class/power_supply");
    if power_supply_dir.exists() {
        if let Ok(entries) = fs::read_dir(power_supply_dir) {
            for entry in entries.filter_map(Result::ok) {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();
                // Check common generic battery identifiers
                if name_str.starts_with("BAT") || name_str.contains("battery") || name_str.starts_with("macsmc-battery") {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}
