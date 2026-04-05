use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Layer, LayerShell};

use crate::config::Config;
use crate::layout;

pub fn create_bar(app: &Application, monitor: &gtk4::gdk::Monitor, config: &Config) -> (ApplicationWindow, gtk4::CssProvider) {
    let window = ApplicationWindow::builder()
        .application(app)
        .build();

    // Initial setup
    window.init_layer_shell();
    window.set_namespace("anomale");
    window.set_layer(Layer::Top);
    window.set_monitor(monitor);
    
    // Anchors - top, left, right (default, unless alignment/width is set)
    let alignment = config.alignment.as_deref().unwrap_or("left"); 
    let position = config.position.as_deref().unwrap_or("top");
    let edge_distance = config.edge_distance.unwrap_or(0);

    
    // Vertical placement
    let vertical_edge = if position == "bottom" {
        gtk4_layer_shell::Edge::Bottom
    } else {
        gtk4_layer_shell::Edge::Top
    };

    window.set_anchor(vertical_edge, true);
    window.set_margin(vertical_edge, edge_distance);

    if let Some(max_width) = config.max_width {
        match alignment {
            "center" => {
                window.set_anchor(gtk4_layer_shell::Edge::Left, false);
                window.set_anchor(gtk4_layer_shell::Edge::Right, false);
            },
            "right" => {
                window.set_anchor(gtk4_layer_shell::Edge::Left, false);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            },
            _ => { // left or anything else
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, false);
            }
        }
        window.set_width_request(max_width);
    } else {
        // Full width behavior
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    }

    // Exclusive zone so windows don't overlap
    window.auto_exclusive_zone_enable();

    // Apply CSS provider for colors and fonts
    let css_provider = gtk4::CssProvider::new();
    let monitor_name = monitor.connector().map(|s| s.to_string());
    let css = generate_css(config, monitor_name.as_deref());
    css_provider.load_from_data(&css);
    
    // Force height request to ensure it shrinks to 16px if content allows
    window.set_height_request(config.bar_height);

    // Try to load user style.css
    if let Ok(config_path) = crate::config::Config::get_config_path(monitor.connector().as_deref()) {
        let style_path = config_path.parent().unwrap().join("style.css");
        if style_path.exists() {
             let user_css_provider = gtk4::CssProvider::new();
             user_css_provider.load_from_path(&style_path);
             gtk4::style_context_add_provider_for_display(
                &gtk4::gdk::Display::default().expect("No display"),
                &user_css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_USER,
            );
        }
    }

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("No display"),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );

    // Create layout
    let content = layout::create_layout(config, monitor);
    window.set_child(Some(&content));

    window.present();
    
    (window, css_provider)
}

pub fn generate_css(config: &Config, monitor_name: Option<&str>) -> String {
    // Sanitize monitor name for CSS class
    let bar_class = monitor_name
        .map(|name| format!(".bar-{}", name.replace("-", "_").replace(":", "_").to_lowercase()))
        .unwrap_or_else(|| "#bar-content".to_string());
    // Calculate effective bar color with opacity if set
    let bar_color = if let Some(opacity) = config.bar_opacity {
        // Clamp opacity between 0 and 100
        let opacity = opacity.max(0).min(100);
        let alpha = (opacity as f64 / 100.0 * 255.0) as u8;
        
        let hex = config.bar_color.trim_start_matches('#');
        let base_hex = if hex.len() >= 6 { &hex[0..6] } else { hex };
        
        format!("#{0}{1:02x}", base_hex, alpha)
    } else {
        config.bar_color.clone()
    };
    
    let position = config.position.as_deref().unwrap_or("top");
    let edge_distance = config.edge_distance.unwrap_or(0);

    // Border Logic
    let border_width_px = config.border_width.unwrap_or(0);
    let border_color = config.border_color.as_deref().unwrap_or("transparent");
    
    let (border_top, border_bottom, border_left, border_right) = if border_width_px > 0 {
        let is_top = position == "top";
        let is_bottom = position == "bottom";
        let alignment = config.alignment.as_deref().unwrap_or("left");
        
        // Top/Bottom borders
        // If anchored to top/bottom (edge_distance == 0), no border on that side.
        let dt = if is_top && edge_distance == 0 { 0 } else { border_width_px };
        let db = if is_bottom && edge_distance == 0 { 0 } else { border_width_px };
        
        let (dl, dr) = if let Some(_) = config.max_width {
            // It has a width limit. Check alignment.
            match alignment {
                "center" => (border_width_px, border_width_px), // Floating in middle
                "left" => (0, border_width_px), // Touching left
                "right" => (border_width_px, 0), // Touching right
                _ => (0, border_width_px) // Default left
            }
        } else {
            // Full width -> touches both sides
            (0, 0)
        };
        
        (dt, db, dl, dr)
    } else {
        (0, 0, 0, 0)
    };

    format!(
        "{19} {{
            --bar-bg: {0};
            --font-family: '{1}';
            --font-color: {2};
            --font-size: {3}px;
            --bar-height: {4}px;
            --radius-tl: {5}px;
            --radius-tr: {6}px;
            --radius-bl: {7}px;
            --radius-br: {8}px;
            --border-width: {12}px {13}px {14}px {15}px;
            --border-color: {16};
        }}
        
        window {{
            background-color: transparent;
        }}

        #bar-content {{
            background-color: var(--bar-bg);
            color: var(--font-color);
            font-family: var(--font-family);
            font-size: var(--font-size);
            min-height: var(--bar-height);
            border-radius: var(--radius-tl) var(--radius-tr) var(--radius-br) var(--radius-bl);
            padding: 0;
            border-style: solid;
            border-width: var(--border-width);
            border-color: var(--border-color);
        }}

        {19} label:not(.dot) {{
            margin-top: {17}px;
            line-height: 1;
        }}

        {19} .dot {{
            margin-top: {18}px;
        }}

        {9}
        {10}
        {11}
        {20}
        {21}",
        bar_color,                                          // 0
        config.font_family,                                 // 1
        config.font_color,                                  // 2
        config.font_size,                                   // 3
        config.bar_height,                                  // 4
        config.border_radius_top_left,                      // 5
        config.border_radius_top_right,                     // 6
        config.border_radius_bottom_left,                   // 7
        config.border_radius_bottom_right,                  // 8
        include_str!("modules/tags/style.css"),              // 9
        include_str!("modules/clock/style.css"),             // 10
        include_str!("modules/volume/style.css"),            // 11
        border_top,                                         // 12
        border_right,                                       // 13
        border_bottom,                                      // 14
        border_left,                                        // 15
        border_color,                                       // 16
        config.font_vert_align,                             // 17
        config.bullet_vert_align,                           // 18
        bar_class,                                          // 19
        include_str!("modules/resources/style.css"),         // 20
        include_str!("modules/battery/style.css")            // 21
    )
}
