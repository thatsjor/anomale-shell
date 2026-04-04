use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use std::sync::{Arc, Mutex};
use sysinfo::{Components, System};
use nvml_wrapper::Nvml;

/// Try to initialize NVML once. Returns None on non-Nvidia systems.
fn get_nvml() -> Option<Arc<Nvml>> {
    thread_local! {
        static NVML_INSTANCE: Option<Arc<Nvml>> = Nvml::init().ok().map(Arc::new);
    }
    NVML_INSTANCE.with(|n| n.clone())
}

pub fn build(resource_type: &str, degree_font: Option<&str>) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("resource-module");
    container.add_css_class(&format!("resource-{}", resource_type));

    let label = Label::new(Some("---"));
    label.set_use_markup(true);
    label.add_css_class("resource-label");
    container.append(&label);

    let label_clone = label.clone();
    let container_clone = container.clone();
    let res_type = resource_type.to_string();
    let deg_font = degree_font.map(|s| s.to_string());

    let sys = Arc::new(Mutex::new(System::new_all()));
    let components = Arc::new(Mutex::new(Components::new_with_refreshed_list()));
    let nvml = get_nvml();

    gtk4::glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let mut sys_lock = sys.lock().unwrap();
        let mut comp_lock = components.lock().unwrap();

        match res_type.as_str() {
            "cpu" => {
                sys_lock.refresh_cpu_usage();
                let usage = sys_lock.global_cpu_info().cpu_usage().round() as u32;

                comp_lock.refresh_list();
                comp_lock.refresh();
                let temp = get_cpu_temp(&comp_lock).round() as u32;

                let deg = format_degree(temp, deg_font.as_deref());
                label_clone.set_markup(&format!("CPU[{}:{:03}%]", deg, usage));
            }
            "gpu" => {
                let usage = get_gpu_usage(nvml.as_deref());

                comp_lock.refresh_list();
                comp_lock.refresh();
                let temp = get_gpu_temp(&comp_lock, nvml.as_deref()).round() as u32;

                let deg = format_degree(temp, deg_font.as_deref());
                label_clone.set_markup(&format!("GPU[{}:{:03}%]", deg, usage));
            }
            "mem" => {
                sys_lock.refresh_memory();
                let total = sys_lock.total_memory();
                let used = sys_lock.used_memory();
                let pct = if total > 0 {
                    (used as f64 / total as f64 * 100.0).round() as u32
                } else {
                    0
                };

                if pct >= 100 {
                    container_clone.add_css_class("resource-oom");
                    label_clone.set_text("MEM[OOM]");
                } else {
                    container_clone.remove_css_class("resource-oom");
                    label_clone.set_text(&format!("MEM[{:02}%]", pct));
                }
            }
            "swap" => {
                sys_lock.refresh_memory();
                let total = sys_lock.total_swap();
                let used = sys_lock.used_swap();
                let pct = if total > 0 {
                    (used as f64 / total as f64 * 100.0).round() as u32
                } else {
                    0
                };
                label_clone.set_text(&format!("SWAP[{:03}%]", pct));
            }
            _ => {
                label_clone.set_text("ERR");
            }
        }

        gtk4::glib::ControlFlow::Continue
    });

    container
}

/// Format a temperature value with the degree symbol.
/// If a custom font is specified, wraps ° in a Pango span.
fn format_degree(temp: u32, font: Option<&str>) -> String {
    match font {
        Some(f) => format!("{:03}<span font_family=\"{}\">°</span>", temp, f),
        None => format!("{:03}°", temp),
    }
}

/// Pick the best CPU temperature sensor.
/// Priority: Tctl > Tdie/Package > core/cpu > k10temp/zenpower.
fn get_cpu_temp(components: &Components) -> f32 {
    let mut best: f32 = 0.0;
    let mut best_priority = 0u8;

    for component in components.list() {
        let name = component.label().to_lowercase();
        let temp = component.temperature();

        if name.contains("tctl") {
            return temp;
        }
        if name.contains("tdie") && best_priority < 3 {
            best = temp;
            best_priority = 3;
        }
        if name.contains("package") && best_priority < 3 {
            best = temp;
            best_priority = 3;
        }
        if (name.contains("core") || name.contains("cpu")) && best_priority < 2 {
            best = temp;
            best_priority = 2;
        }
        if (name.contains("k10temp") || name.contains("zenpower")) && best_priority < 1 {
            best = temp;
            best_priority = 1;
        }
    }

    best
}

/// Get GPU temperature.
/// 1. sysinfo components (AMD/Intel via hwmon)
/// 2. NVML (Nvidia — in-process, no subprocess)
fn get_gpu_temp(components: &Components, nvml: Option<&Nvml>) -> f32 {
    for component in components.list() {
        let name = component.label().to_lowercase();
        if name.contains("gpu") || name.contains("amdgpu") || name.contains("edge") || name.contains("nouveau") {
            return component.temperature();
        }
    }

    if let Some(nvml) = nvml {
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                return temp as f32;
            }
        }
    }

    0.0
}

/// Get GPU usage percentage.
/// 1. /sys/class/drm (AMD/Intel kernel sysfs)
/// 2. NVML (Nvidia — in-process, no subprocess)
fn get_gpu_usage(nvml: Option<&Nvml>) -> u32 {
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path().join("device/gpu_busy_percent");
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(val) = content.trim().parse::<u32>() {
                        return val;
                    }
                }
            }
        }
    }

    if let Some(nvml) = nvml {
        if let Ok(device) = nvml.device_by_index(0) {
            if let Ok(utilization) = device.utilization_rates() {
                return utilization.gpu;
            }
        }
    }

    0
}
