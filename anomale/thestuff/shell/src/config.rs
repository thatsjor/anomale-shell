use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub bar_height: i32,
    pub bar_color: String,
    pub font_family: String,
    pub font_size: f64,
    pub font_color: String,
    pub clock_format: String,
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
    pub max_width: Option<i32>,
    pub alignment: Option<String>,
    pub border_radius_top_left: i32,
    pub border_radius_top_right: i32,
    pub border_radius_bottom_left: i32,
    pub border_radius_bottom_right: i32,
    pub position: Option<String>,
    pub edge_distance: Option<i32>,
    pub pywal: bool,
    pub bar_opacity: Option<i32>,
    pub border_width: Option<i32>,
    pub border_color: Option<String>,
    pub exec: Vec<String>,
    pub exec_once: Vec<String>,
    pub font_vert_align: i32,
    pub bullet_vert_align: i32,
    pub volume_scroll_speed: f64,
    pub degree_symbol_font: Option<String>,
    pub charge_color: String,
    pub shadow_size: i32,
    pub shadow_blur: i32,
    pub shadow_offset_x: i32,
    pub shadow_offset_y: i32,
    pub shadow_color: String,
    pub shadow_opacity: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bar_height: 30,
            bar_color: "#1e1e2e".to_string(),
            font_family: "Sans".to_string(),
            font_size: 14.0,
            font_color: "#cdd6f4".to_string(),
            clock_format: "%H:%M".to_string(),
            left: vec!["tags".to_string()],
            center: vec!["clock".to_string()],
            right: vec![],
            max_width: None,
            alignment: None,
            border_radius_top_left: 0,
            border_radius_top_right: 0,
            border_radius_bottom_left: 0,
            border_radius_bottom_right: 0,
            position: None,
            edge_distance: None,
            pywal: false,
            bar_opacity: None,
            border_width: None,
            border_color: None,
            exec: vec![],
            exec_once: vec![],
            font_vert_align: 2,
            bullet_vert_align: 0,
            volume_scroll_speed: 5.0,
            degree_symbol_font: None,
            charge_color: "#00ff00ff".to_string(),
            shadow_size: 0,
            shadow_blur: 0,
            shadow_offset_x: 0,
            shadow_offset_y: 0,
            shadow_color: "#00000040".to_string(),
            shadow_opacity: 1.0,
        }
    }
}

impl Config {
    pub fn load(monitor_name: Option<&str>) -> Result<Self> {
        let mut config = Self::default();

        // 1. Always load global exec/exec-once from default config
        if let Ok(default_path) = Self::get_config_path(None) {
             if default_path.exists() {
                 let content = fs::read_to_string(&default_path)
                     .context(format!("Failed to read default config file: {:?}", default_path))?;
                 config.apply_execs(&content);
             }
        }

        // 2. Determine which config file to use for settings
        // If a monitor-specific config exists, use ONLY that for settings.
        // Otherwise, use the default config for settings.
        let target_path = if let Some(name) = monitor_name {
            let specific_path = Self::get_config_path(Some(name))?;
            if specific_path.exists() {
                Some(specific_path)
            } else {
                // Fallback to default if specific doesn't exist
                 Self::get_config_path(None).ok().filter(|p| p.exists())
            }
        } else {
             Self::get_config_path(None).ok().filter(|p| p.exists())
        };

        if let Some(path) = &target_path {
            println!("DEBUG: Loading config for {:?} from path: {:?}", monitor_name, path);
            let content = fs::read_to_string(path)
                .context(format!("Failed to read config file: {:?}", path))?;
            config.apply_all(&content);
        } else {
            println!("DEBUG: No config found for {:?}, using default values", monitor_name);
        }

        Ok(config)
    }

    pub fn get_config_path(monitor_name: Option<&str>) -> Result<PathBuf> {
        let config_filename = if let Some(name) = monitor_name {
            format!("{}.conf", name)
        } else {
            "config.conf".to_string()
        };

        // Check current directory first for development convenience (only for default config or specific if present)
        let local_path = PathBuf::from(&config_filename);
        if local_path.exists() {
            println!("DEBUG: Found local config: {:?}", local_path);
            return Ok(fs::canonicalize(local_path)?);
        }

        let xdg_config = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".config"))
            })
            .context("Could not determine config directory")?;
        
        let full_path = xdg_config.join("anomale").join(&config_filename);
        println!("DEBUG: Checking config at: {:?}", full_path);
        
        Ok(full_path)
    }

    pub fn load_pywal_colors() -> Option<HashMap<String, String>> {
        let home = std::env::var("HOME").ok()?;
        let path = PathBuf::from(home).join(".cache/wal/colors.json");
        
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        
        let colors = json.get("colors")?.as_object()?;
        
        let mut map = HashMap::new();
        for (k, v) in colors {
            if let Some(hex) = v.as_str() {
                map.insert(k.clone(), hex.to_string());
            }
        }
        Some(map)
    }

    pub fn resolve_color(value: &str, pywal_colors: &Option<HashMap<String, String>>) -> String {
        if value.starts_with("pywal_color") {
             if let Some(colors) = pywal_colors {
                 // extract the color name, e.g., "pywal_color0" -> "color0"
                 // User format: "pywal_color0"
                 // Json keys: "color0"
                 let key = value.replace("pywal_", "");
                 if let Some(hex) = colors.get(&key) {
                     return format!("{}ff", hex); // Append alpha if missing? Pywal usually gives #RRGGBB. We might need #RRGGBBAA. GTK/CSS accepts #RRGGBB too.
                 }
             }
        }
        value.to_string()
    }

    /// Apply an opacity multiplier (0.0–1.0) to a hex color string.
    /// Handles #RGB, #RRGGBB, and #RRGGBBAA formats.
    /// Returns #RRGGBBAA with alpha = existing_alpha * opacity.
    pub fn apply_opacity_to_hex(hex: &str, opacity: f64) -> String {
        let h = hex.trim_start_matches('#');
        let (r, g, b, a) = match h.len() {
            3 => {
                let r = u8::from_str_radix(&h[0..1], 16).unwrap_or(0) * 17;
                let g = u8::from_str_radix(&h[1..2], 16).unwrap_or(0) * 17;
                let b = u8::from_str_radix(&h[2..3], 16).unwrap_or(0) * 17;
                (r, g, b, 255u8)
            }
            6 => {
                let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
                (r, g, b, 255u8)
            }
            8 => {
                let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&h[6..8], 16).unwrap_or(255);
                (r, g, b, a)
            }
            _ => return hex.to_string(), // Can't parse, return as-is
        };
        let new_alpha = ((a as f64) * opacity.clamp(0.0, 1.0)).round() as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, new_alpha)
    }

    fn apply_execs(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "exec" => self.exec.push(value.to_string()),
                    "exec-once" => self.exec_once.push(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    fn apply_all(&mut self, content: &str) {
        let mut properties = HashMap::new();

        // Parse lines
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "exec" => self.exec.push(value.to_string()),
                    "exec-once" => self.exec_once.push(value.to_string()),
                    _ => { properties.insert(key, value); }
                }
            }
        }

        // Apply properties
        if let Some(val) = properties.get("pywal") {
            if val.eq_ignore_ascii_case("true") {
                self.pywal = true;
            }
        }

        let pywal_colors = if self.pywal {
            Self::load_pywal_colors()
        } else {
            None
        };

        if let Some(val) = properties.get("bar_height") {
            if let Ok(v) = val.parse() { self.bar_height = v; }
        }
        if let Some(val) = properties.get("bar_color") {
            self.bar_color = Self::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("font_family") {
            self.font_family = val.to_string();
        }
        if let Some(val) = properties.get("font_size") {
            if let Ok(v) = val.parse() { self.font_size = v; }
        }
        if let Some(val) = properties.get("font_color") {
            self.font_color = Self::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("clock_format") {
            self.clock_format = val.trim_matches('"').to_string();
        }
        if let Some(val) = properties.get("left") {
            self.left = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(val) = properties.get("center") {
            self.center = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(val) = properties.get("right") {
            self.right = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        }
        if let Some(val) = properties.get("max_width") {
            if let Ok(v) = val.parse() { self.max_width = Some(v); }
        }
        if let Some(val) = properties.get("alignment") {
            self.alignment = Some(val.to_string());
        }
        if let Some(val) = properties.get("tleft_corner") {
            if let Ok(v) = val.parse() { self.border_radius_top_left = v; }
        }
        if let Some(val) = properties.get("tright_corner") {
            if let Ok(v) = val.parse() { self.border_radius_top_right = v; }
        }
        if let Some(val) = properties.get("bleft_corner") {
            if let Ok(v) = val.parse() { self.border_radius_bottom_left = v; }
        }
        if let Some(val) = properties.get("bright_corner") {
            if let Ok(v) = val.parse() { self.border_radius_bottom_right = v; }
        }
        if let Some(val) = properties.get("position") {
            self.position = Some(val.to_string());
        }
        if let Some(val) = properties.get("edge_distance") {
            if let Ok(v) = val.parse() { self.edge_distance = Some(v); }
        }
        if let Some(val) = properties.get("bar_opacity") {
            if let Ok(v) = val.parse() { self.bar_opacity = Some(v); }
        }
        if let Some(val) = properties.get("border") {
            if let Ok(v) = val.parse() { self.border_width = Some(v); }
        }
        if let Some(val) = properties.get("border_color") {
            self.border_color = Some(Self::resolve_color(val, &pywal_colors));
        }
        if let Some(val) = properties.get("font_vert_align") {
            if let Ok(v) = val.parse() { self.font_vert_align = v; }
        }
        if let Some(val) = properties.get("bullet_vert_align") {
            if let Ok(v) = val.parse() { self.bullet_vert_align = v; }
        }
        if let Some(val) = properties.get("volume_scroll_speed") {
            if let Ok(v) = val.parse() { self.volume_scroll_speed = v; }
        }
        if let Some(val) = properties.get("degree_symbol_font") {
            self.degree_symbol_font = Some(val.to_string());
        }
        if let Some(val) = properties.get("charge_color") {
            self.charge_color = Self::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("shadow_size") {
            if let Ok(v) = val.parse() { self.shadow_size = v; }
        }
        if let Some(val) = properties.get("shadow_blur") {
            if let Ok(v) = val.parse() { self.shadow_blur = v; }
        }
        if let Some(val) = properties.get("shadow_offset_x") {
            if let Ok(v) = val.parse() { self.shadow_offset_x = v; }
        }
        if let Some(val) = properties.get("shadow_offset_y") {
            if let Ok(v) = val.parse() { self.shadow_offset_y = v; }
        }
        if let Some(val) = properties.get("shadow_color") {
            self.shadow_color = Self::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("shadow_opacity") {
            if let Ok(v) = val.parse::<f64>() {
                self.shadow_opacity = v.clamp(0.0, 1.0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub search_width: i32,
    pub max_results_height: i32,
    pub background_color: String,
    pub border_color: String,
    pub border_width: i32,
    pub border_radius: i32,
    pub text_color: String,
    pub font_family: String,
    pub font_size: f64,
    pub selection_color: String,
    pub selection_text_color: String,
    pub pywal: bool,
    pub background_opacity: f64,
    pub apps_opacity: Option<f64>,
    pub power_opacity: Option<f64>,
    pub wallpapers_opacity: Option<f64>,
    pub list_text_color: String,
    pub highlight_color: String,
    pub highlight_text_color: String,
    pub window_namespace: String,
    pub power_actions: Vec<(String, String)>,
    pub wallpapers_path: String,
    pub wallpapers_thumb_size: i32,
    pub wallpapers_command: String,
    pub use_last_wall: bool,
    pub wallpapers_width: i32,
    pub wallpapers_height: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            search_width: 600,
            max_results_height: 400,
            background_color: "#1e1e2eff".to_string(),
            border_color: "#cba6f7ff".to_string(),
            border_width: 2,
            border_radius: 12,
            text_color: "#cdd6f4ff".to_string(),
            font_family: "Sans".to_string(),
            font_size: 14.0,
            selection_color: "#313244ff".to_string(),
            selection_text_color: "#cdd6f4ff".to_string(),
            pywal: false,
            background_opacity: 1.0,
            apps_opacity: None,
            power_opacity: None,
            wallpapers_opacity: None,
            list_text_color: "#cdd6f4ff".to_string(),
            highlight_color: "#313244ff".to_string(),
            highlight_text_color: "#cdd6f4ff".to_string(),
            window_namespace: "anomale-launcher".to_string(),
            power_actions: vec![
                ("Reboot".to_string(), "systemctl reboot".to_string()),
                ("Shutdown".to_string(), "systemctl poweroff".to_string()),
                ("Logout".to_string(), "mmsg -d quit".to_string()),
            ],
            wallpapers_path: "~/Pictures/wallpapers/".to_string(),
            wallpapers_thumb_size: 200,
            wallpapers_command: "wal --backend haishoku -i [[w]]".to_string(),
            use_last_wall: false,
            wallpapers_width: 800,
            wallpapers_height: 600,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        let xdg_config = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".config"))
            })
            .context("Could not determine config directory")?;
        
        // 1. Check XDG config
        let config_path = xdg_config.join("anomale").join("menus.conf");
        println!("DEBUG: Checking menus config at: {:?}", config_path);

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context(format!("Failed to read apps config file: {:?}", config_path))?;
            config.apply_all(&content);
            return Ok(config);
        }

        // 2. Check local path (CWD)
        let local_path = PathBuf::from("menus.conf");
        if local_path.exists() {
             println!("DEBUG: Found local menus.conf at {:?}", local_path);
             let content = fs::read_to_string(&local_path)?;
             config.apply_all(&content);
             return Ok(config);
        }

        // 3. Check relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            // Resolve symlinks to find the real location
            if let Ok(real_exe_path) = fs::canonicalize(&exe_path) {
                if let Some(parent) = real_exe_path.parent() {
                    // Try to find project root (assuming executable is in target/release/ or similar)
                    // Strategy: Search up directories for menus.conf
                    let mut current_dir = parent;
                    for _ in 0..5 { // Search up to 5 levels
                        let probe = current_dir.join("menus.conf");
                        if probe.exists() {
                            println!("DEBUG: Found menus.conf relative to executable at {:?}", probe);
                            let content = fs::read_to_string(&probe)?;
                            config.apply_all(&content);
                            return Ok(config);
                        }
                        if let Some(p) = current_dir.parent() {
                            current_dir = p;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        
        println!("DEBUG: No menus.conf found. Using defaults.");
        Ok(config)
    }

    fn apply_all(&mut self, content: &str) {
        let mut properties = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                properties.insert(key.trim(), value.trim());
            }
        }

        if let Some(val) = properties.get("pywal") {
            if val.eq_ignore_ascii_case("true") {
                self.pywal = true;
            }
        } else {
            // Auto-detect pywal usage if not explicitly set
            for value in properties.values() {
                if value.contains("pywal_") {
                    self.pywal = true;
                    break;
                }
            }
        }

        let pywal_colors = if self.pywal {
            Config::load_pywal_colors()
        } else {
            None
        };

        if let Some(val) = properties.get("search_width") {
            if let Ok(v) = val.parse() { self.search_width = v; }
        }
        if let Some(val) = properties.get("max_results_height") {
            if let Ok(v) = val.parse() { self.max_results_height = v; }
        }
        if let Some(val) = properties.get("background_color") {
            self.background_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("border_color") {
            self.border_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("border_width") {
             if let Ok(v) = val.parse() { self.border_width = v; }
        }
        if let Some(val) = properties.get("border_radius") {
             if let Ok(v) = val.parse() { self.border_radius = v; }
        }
        if let Some(val) = properties.get("text_color") {
            self.text_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("font_family") {
            self.font_family = val.to_string();
        }
        if let Some(val) = properties.get("font_size") {
             if let Ok(v) = val.parse() { self.font_size = v; }
        }
        if let Some(val) = properties.get("selection_color") {
            self.selection_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("selection_text_color") {
            self.selection_text_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("list_text_color") {
            self.list_text_color = Config::resolve_color(val, &pywal_colors);
        } else {
            // Default to text_color if not specified
            self.list_text_color = self.text_color.clone();
        }
        if let Some(val) = properties.get("highlight_color") {
            self.highlight_color = Config::resolve_color(val, &pywal_colors);
        } else {
             self.highlight_color = self.selection_color.clone();
        }
        if let Some(val) = properties.get("highlight_text_color") {
            self.highlight_text_color = Config::resolve_color(val, &pywal_colors);
        } else {
             self.highlight_text_color = self.selection_text_color.clone();
        }
        if let Some(val) = properties.get("background_opacity") {
            if let Ok(v) = val.parse() { self.background_opacity = v; }
        }
        if let Some(val) = properties.get("apps_opacity") {
            if let Ok(v) = val.parse() { self.apps_opacity = Some(v); }
        }
        if let Some(val) = properties.get("power_opacity") {
            if let Ok(v) = val.parse() { self.power_opacity = Some(v); }
        }
        if let Some(val) = properties.get("wallpapers_opacity") {
            if let Ok(v) = val.parse() { self.wallpapers_opacity = Some(v); }
        }
        if let Some(val) = properties.get("window_namespace") {
            self.window_namespace = val.to_string();
        }
        if let Some(val) = properties.get("wallpapers_path") {
            self.wallpapers_path = val.to_string();
        }
        if let Some(val) = properties.get("wallpapers_thumb_size") {
            if let Ok(v) = val.parse() { self.wallpapers_thumb_size = v; }
        }
        if let Some(val) = properties.get("wallpapers_command") {
            self.wallpapers_command = val.to_string();
        }
        if let Some(val) = properties.get("use_last_wall") {
            self.use_last_wall = val.eq_ignore_ascii_case("true");
        }
        if let Some(val) = properties.get("wallpapers_size") {
            if let Some((w, h)) = val.split_once('x') {
                if let (Ok(width), Ok(height)) = (w.trim().parse(), h.trim().parse()) {
                    self.wallpapers_width = width;
                    self.wallpapers_height = height;
                }
            }
        }

        let mut power_entries: Vec<(&str, &str)> = properties.iter()
            .filter(|(k, _)| k.starts_with("power_") && **k != "power_opacity")
            .map(|(k, v)| (*k, *v))
            .collect();
        
        // Sort by key (power_1, power_2, etc.)
        power_entries.sort_by_key(|(k, _)| *k);

        if !power_entries.is_empty() {
             self.power_actions.clear();
             for (_, val) in power_entries {
                 if let Some((label, cmd)) = val.split_once(':') {
                     self.power_actions.push((label.trim().to_string(), cmd.trim().to_string()));
                 } else {
                     // If no colon, treat value as both label and command? Or just skip?
                     // Let's assume label=cmd if no colon
                     self.power_actions.push((val.trim().to_string(), val.trim().to_string()));
                 }
             }
        }

        // Sanitize values to prevent crashes
        if self.wallpapers_thumb_size < 50 { self.wallpapers_thumb_size = 50; }
        if self.wallpapers_width < 100 { self.wallpapers_width = 100; }
        if self.wallpapers_height < 100 { self.wallpapers_height = 100; }
        if self.search_width < 100 { self.search_width = 100; }
        if self.max_results_height < 50 { self.max_results_height = 50; }
    }

    pub fn generate_css(&self, bar_config: Option<&Config>) -> String {
        // Generate wallpaper-window shadow CSS from bar config if available
        let wallpaper_shadow_css = if let Some(bc) = bar_config {
            let has_shadow = bc.shadow_size > 0 || bc.shadow_blur > 0
                || bc.shadow_offset_x != 0 || bc.shadow_offset_y != 0;
            if has_shadow {
                let effective_color = Config::apply_opacity_to_hex(&bc.shadow_color, bc.shadow_opacity);
                // Margin so the shadow has room to render inside the transparent window
                let margin_top = 0i32.max(bc.shadow_size - bc.shadow_offset_y) + bc.shadow_blur;
                let margin_bottom = 0i32.max(bc.shadow_size + bc.shadow_offset_y) + bc.shadow_blur;
                let margin_left = 0i32.max(bc.shadow_size - bc.shadow_offset_x) + bc.shadow_blur;
                let margin_right = 0i32.max(bc.shadow_size + bc.shadow_offset_x) + bc.shadow_blur;
                format!(
                    "\n            .wallpaper-window {{\n                box-shadow: {}px {}px {}px {}px {};\n                margin: {}px {}px {}px {}px;\n            }}",
                    bc.shadow_offset_x,
                    bc.shadow_offset_y,
                    bc.shadow_blur,
                    bc.shadow_size,
                    effective_color,
                    margin_top, margin_right, margin_bottom, margin_left
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let apps_op = self.apps_opacity.unwrap_or(self.background_opacity);
        let power_op = self.power_opacity.unwrap_or(self.background_opacity);
        let wall_op = self.wallpapers_opacity.unwrap_or(self.background_opacity);

        format!(
            "
            .apps-window {{
                background-color: alpha({bg}, {apps_op});
            }}

            .power-window {{
                background-color: alpha({bg}, {power_op});
            }}

            .wallpaper-window {{
                background-color: alpha({bg}, {wall_op});
                border: {bw}px solid {bc};
                border-radius: {br}px;
            }}
    
            .launcher-box {{
                background-color: transparent;
                padding: 10px;
                font-family: '{font}';
                font-size: {fsize}px;
                color: {fg};
            }}
    
            scrollbar {{
                background-color: transparent;
                border: none;
            }}
    
            scrollbar trough {{
                background-color: transparent;
                border: none;
            }}
    
            scrollbar slider {{
                background-color: {bc};
                min-width: {bw}px; 
                min-height: 20px;
                border-radius: {br}px; 
                margin: 2px;
                border: none;
            }}
            
            .search-entry, .search-entry > text, .search-entry entry {{
                background-color: transparent;
                color: {fg};
                border: none;
                box-shadow: none;
                outline: none;
                border-radius: 0px;
                padding: 5px;
                margin-right: 10px;
            }}
            
            .search-entry.has-results {{
                border-bottom: 2px solid {bc};
            }}
            
            .search-entry:focus {{
                outline: none;
                box-shadow: none;
            }}
            
            .search-entry.has-results:focus {{
                 border-bottom: 2px solid {sel};
            }}
    
            .search-entry selection {{
                background-color: {hl};
                color: {hlfg};
            }}
    
            .app-list {{
                background-color: transparent;
                outline: none;
                margin-top: 10px;
            }}
    
            .app-icon {{
                margin-left: 5px;
            }}
    
            row {{
                padding: 5px;
                border-radius: {br}px;
                outline: none;
                color: {list_fg};
                margin-right: 10px;
            }}
            
            row:focus {{
                 outline: none;
            }}
    
            row:selected {{
                background-color: {sel};
                color: {hlfg};
                outline: none;
            }}
    
            row:hover {{
                 background-color: alpha({sel}, 0.5);
                 color: {hlfg};
            }}
    
            flowboxchild {{
                padding: 5px;
                border-radius: {br}px;
                outline: none;
                transition: background-color 150ms ease;
            }}
    
            flowboxchild:selected {{
                background-color: {sel};
                outline: none;
            }}
    
            flowboxchild:hover {{
                background-color: alpha({sel}, 0.3);
            }}
    
            flowboxchild:focus {{
                outline: none;
            }}

            .wallpaper-thumb {{
                padding: 5px;
                border-radius: {br}px;
                transition: background-color 150ms ease;
            }}

            .wallpaper-thumb:hover {{
                background-color: alpha({sel}, 0.3);
            }}

            .wallpaper-thumb.selected {{
                background-color: {sel};
            }}

            {wallpaper_shadow}
            ",
            bg = self.background_color,
            bw = self.border_width,
            bc = self.border_color,
            br = self.border_radius,
            font = self.font_family,
            fsize = self.font_size,
            fg = self.text_color,
            sel = self.selection_color,
            list_fg = self.list_text_color,
            hl = self.highlight_color,
            hlfg = self.highlight_text_color,
            wallpaper_shadow = wallpaper_shadow_css
        )
    }
}


#[derive(Debug, Clone)]
pub struct NotifyConfig {
    pub width: i32,
    pub height: i32,
    pub margin: i32,
    pub spacing: i32,
    pub corner: String,
    pub timeout: i32,
    pub pywal: bool,
    pub background_color: String,
    pub background_opacity: f64,
    pub border_color: String,
    pub border_width: i32,
    pub border_radius: i32,
    pub text_color: String,
    pub font_family: String,
    pub font_size: f64,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            width: 350,
            height: 100,
            margin: 20,
            spacing: 10,
            corner: "bottom-right".to_string(),
            timeout: 2,
            pywal: false,
            background_color: "#1e1e2eff".to_string(),
            background_opacity: 0.9,
            border_color: "#cba6f7ff".to_string(),
            border_width: 2,
            border_radius: 12,
            text_color: "#cdd6f4ff".to_string(),
            font_family: "Sans".to_string(),
            font_size: 12.0,
        }
    }
}

impl NotifyConfig {
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        let xdg_config = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".config"))
            })
            .context("Could not determine config directory")?;
        
        let config_path = xdg_config.join("anomale").join("notifications.conf");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            config.apply_all(&content);
            return Ok(config);
        }

        let local_path = PathBuf::from("notifications.conf");
        if local_path.exists() {
             let content = fs::read_to_string(&local_path)?;
             config.apply_all(&content);
             return Ok(config);
        }

        Ok(config)
    }

    fn apply_all(&mut self, content: &str) {
        let mut properties = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                // Strip inline comments (e.g. "bottom-left # options...")
                let value = value.split('#').next().unwrap_or("").trim();
                properties.insert(key.trim(), value);
            }
        }

        if let Some(val) = properties.get("pywal") {
            self.pywal = val.eq_ignore_ascii_case("true");
        } else {
            for value in properties.values() {
                if value.contains("pywal_") {
                    self.pywal = true;
                    break;
                }
            }
        }

        let pywal_colors = if self.pywal {
            Config::load_pywal_colors()
        } else {
            None
        };

        if let Some(val) = properties.get("width") {
            if let Ok(v) = val.parse() { self.width = v; }
        }
        if let Some(val) = properties.get("height") {
            if let Ok(v) = val.parse() { self.height = v; }
        }
        if let Some(val) = properties.get("margin") {
            if let Ok(v) = val.parse() { self.margin = v; }
        }
        if let Some(val) = properties.get("spacing") {
            if let Ok(v) = val.parse() { self.spacing = v; }
        }
        if let Some(val) = properties.get("corner") {
            self.corner = val.to_string();
        }
        if let Some(val) = properties.get("timeout") {
            if let Ok(v) = val.parse() { self.timeout = v; }
        }
        if let Some(val) = properties.get("background_color") {
            self.background_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("background_opacity") {
            if let Ok(v) = val.parse() { self.background_opacity = v; }
        }
        if let Some(val) = properties.get("border_color") {
            self.border_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("border_width") {
            if let Ok(v) = val.parse() { self.border_width = v; }
        }
        if let Some(val) = properties.get("border_radius") {
            if let Ok(v) = val.parse() { self.border_radius = v; }
        }
        if let Some(val) = properties.get("text_color") {
            self.text_color = Config::resolve_color(val, &pywal_colors);
        }
        if let Some(val) = properties.get("font_family") {
            self.font_family = val.to_string();
        }
        if let Some(val) = properties.get("font_size") {
            if let Ok(v) = val.parse() { self.font_size = v; }
        }
    }

    pub fn generate_css(&self) -> String {
        let bg_hex = self.background_color.trim_start_matches('#');
        let (r, g, b) = if bg_hex.len() >= 6 {
            (
                u32::from_str_radix(&bg_hex[0..2], 16).unwrap_or(0),
                u32::from_str_radix(&bg_hex[2..4], 16).unwrap_or(0),
                u32::from_str_radix(&bg_hex[4..6], 16).unwrap_or(0),
            )
        } else {
            (0, 0, 0)
        };

        // Convert any hex color to rgba() format that GTK4's CSS engine reliably handles.
        // #RRGGBBAA is not well-supported; rgba() always works.
        let to_rgba = |hex: &str| -> String {
            let h = hex.trim_start_matches('#');
            match h.len() {
                6 => {
                    let rv = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
                    let gv = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
                    let bv = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
                    format!("rgba({}, {}, {}, 1.0)", rv, gv, bv)
                }
                8 => {
                    let rv = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
                    let gv = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
                    let bv = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
                    let av = u8::from_str_radix(&h[6..8], 16).unwrap_or(255);
                    format!("rgba({}, {}, {}, {})", rv, gv, bv, av as f64 / 255.0)
                }
                _ => hex.to_string(),
            }
        };

        format!(
            "
            .anomale-notification-window {{
                background-color: transparent;
            }}

            .notification-window {{
                background-color: rgba({r}, {g}, {b}, {opacity});
                border: {bw}px solid {bc};
                border-radius: {br}px;
                color: {fg};
                font-family: '{font}';
                font-size: {fsize}px;
                padding: 15px;
            }}

            .notification-summary {{
                font-weight: bold;
                font-size: {summary_size}px;
                margin-bottom: 2px;
            }}


            .notification-body {{
                opacity: 0.9;
            }}
            ",
            r = r,
            g = g,
            b = b,
            opacity = self.background_opacity,
            bw = self.border_width,
            bc = to_rgba(&self.border_color),
            br = self.border_radius,
            fg = to_rgba(&self.text_color),
            font = self.font_family,
            fsize = self.font_size,
            summary_size = self.font_size + 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_config_loading_logic() -> Result<()> {
        // Change CWD to a temp dir to avoid picking up local config.conf
        let cwd_temp = TempDir::new()?;
        std::env::set_current_dir(&cwd_temp)?;

        // Create a temp directory for configs
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join("anomale");
        fs::create_dir_all(&config_dir)?;

        // Override XDG_CONFIG_HOME to point to temp dir
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        // Create global config
        let global_config_path = config_dir.join("config.conf");
        let mut global_file = fs::File::create(&global_config_path)?;
        writeln!(global_file, "bar_height=10")?;
        writeln!(global_file, "exec=GLOBAL_EXEC")?;
        writeln!(global_file, "bar_color=#000000")?;

        // Create specific config
        let specific_config_path = config_dir.join("HDMI-TEST.conf");
        let mut specific_file = fs::File::create(&specific_config_path)?;
        writeln!(specific_file, "bar_height=20")?;
        writeln!(specific_file, "exec=LOCAL_EXEC")?;
        // bar_color not specified, should remain default (NOT global)
        
        let config = Config::load(Some("HDMI-TEST"))?;
        
        assert_eq!(config.bar_height, 20, "Specific config should override height");
        assert!(config.exec.contains(&"GLOBAL_EXEC".to_string()), "Should contain global execs");
        assert!(config.exec.contains(&"LOCAL_EXEC".to_string()), "Should contain local execs");
        assert_ne!(config.bar_color, "#000000", "Specific config should NOT inherit global visual settings");
        assert_eq!(config.bar_color, "#1e1e2e", "Specific config should use defaults if missing");

 
        
        let config_fallback = Config::load(Some("DP-TEST"))?;
        
        assert_eq!(config_fallback.bar_height, 10, "Fallback should use output from config.conf");
        assert!(config_fallback.exec.contains(&"GLOBAL_EXEC".to_string()));
        assert_eq!(config_fallback.bar_color, "#000000");

        Ok(())
    }
    #[test]
    fn test_app_config_pywal_auto_detection() -> Result<()> {
        // Change CWD to a temp dir
        let cwd_temp = TempDir::new()?;
        std::env::set_current_dir(&cwd_temp)?;

        // Create a temp directory for configs
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join("anomale");
        fs::create_dir_all(&config_dir)?;

        // Override XDG_CONFIG_HOME to point to temp dir
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        // Create apps.conf with pywal colors but NO pywal=true
        let apps_config_path = config_dir.join("apps.conf");
        let mut file = fs::File::create(&apps_config_path)?;
        writeln!(file, "background_color=pywal_color0")?;
        // writeln!(file, "border_color=pywal_color1")?; // Optional
        writeln!(file, "background_opacity=0.8")?;
        writeln!(file, "list_text_color=#aabbcc")?;
        writeln!(file, "highlight_color=#112233")?;

        // Load config
        // access AppConfig via super
        let config = AppConfig::load()?;

        // Verify pywal was auto-detected
        assert!(config.pywal, "Pywal should be auto-detected when pywal_color is used");
        
        // Verify opacity was parsed
        assert_eq!(config.background_opacity, 0.8, "Background opacity should be 0.8");
        
        // Verify list text color
        assert_eq!(config.list_text_color, "#aabbcc", "List text color should be parsed correctly");
        
        // Verify highlight color
        assert_eq!(config.highlight_color, "#112233", "Highlight color should be parsed correctly");
        
        // Verify default namespace (since we didn't set it in the file)
        assert_eq!(config.window_namespace, "anomale-launcher", "Default namespace should be used");

        Ok(())
    }
}
