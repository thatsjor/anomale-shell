use gtk4::prelude::*;
use gtk4::{Box, CenterBox, Orientation};

use crate::config::Config;
use crate::modules;

pub fn create_layout(config: &Config, monitor: &gtk4::gdk::Monitor) -> CenterBox {
    let container = CenterBox::new();
    container.set_widget_name("bar-content");
    
    // Add monitor-specific CSS class for per-bar styling
    if let Some(monitor_name) = monitor.connector() {
        // Sanitize monitor name for CSS class (replace special chars)
        let safe_name = monitor_name.replace("-", "_").replace(":", "_").to_lowercase();
        container.add_css_class(&format!("bar-{}", safe_name));
    }

    let left_box = Box::new(Orientation::Horizontal, 10);
    let center_box = Box::new(Orientation::Horizontal, 10);
    let right_box = Box::new(Orientation::Horizontal, 10);

    // Apply some padding
    left_box.set_margin_start(10);
    left_box.set_margin_end(10);
    center_box.set_margin_start(10);
    center_box.set_margin_end(10);
    right_box.set_margin_start(10);
    right_box.set_margin_end(10);

    populate_box(&left_box, &config.left, config, monitor);
    populate_box(&center_box, &config.center, config, monitor);
    populate_box(&right_box, &config.right, config, monitor);

    container.set_start_widget(Some(&left_box));
    container.set_center_widget(Some(&center_box));
    container.set_end_widget(Some(&right_box));

    container
}

fn populate_box(bbox: &Box, modules: &[String], config: &Config, monitor: &gtk4::gdk::Monitor) {
    for module_name in modules {
        match module_name.as_str() {
            "tags" => {
                let widget = modules::tags::build(monitor);
                bbox.append(&widget);
            },
            "clock" => {
                let widget = modules::clock::build(config.clock_format.clone());
                bbox.append(&widget);
            },
            "volume" => {
                let widget = modules::volume::build(config.volume_scroll_speed);
                bbox.append(&widget);
            },
            "battery" => {
                let widget = modules::battery::build(config.charge_color.clone());
                bbox.append(&widget);
            },
            name if name.starts_with("resource:") || name.starts_with("resources:") => {
                let parts: Vec<&str> = name.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let widget = modules::resources::build(parts[1].trim(), config.degree_symbol_font.as_deref());
                    bbox.append(&widget);
                }
            },
            _ => eprintln!("Unknown module: {}", module_name),
        }
    }
}
