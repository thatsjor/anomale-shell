use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, ListBox, ListBoxRow, Orientation, Align, SelectionMode};
use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};
use crate::config::AppConfig;
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;

pub struct PowerMenu {
    pub window: ApplicationWindow,
    pub list_box: ListBox,
    css_provider: gtk4::CssProvider,
}

impl PowerMenu {
    pub fn new(app: &Application, css_provider_ref: &gtk4::CssProvider) -> Rc<RefCell<Self>> {
        let config = AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load apps config: {}. Using defaults.", e);
            AppConfig::default()
        });

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Anomale Power Menu")
            .decorated(false)
            .visible(false) // Initially hidden
            .build();

        // Layer Shell Setup - Full screen overlay
        window.init_layer_shell();
        window.set_namespace(&config.window_namespace);
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_exclusive_zone(-1); // Cover everything including the bar
        
        // Anchor all edges for full-screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Apply Shared CSS (Initial load)
        let css = config.generate_css();
        css_provider_ref.load_from_data(&css);
        let css_provider = css_provider_ref.clone();

        // Full-screen overlay container
        let overlay_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();
        window.add_css_class("fullscreen-bg");

        // Inner launcher box (using same class for consistency)
        let launcher_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();
        // Use same width + scrollbar gutter logic as apps menu
        launcher_box.set_width_request(config.search_width + 10);
        launcher_box.add_css_class("launcher-box");

        // Actions List
        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();
        list_box.add_css_class("app-list");

        // Populate List
        for (label_text, cmd) in &config.power_actions {
            let row = ListBoxRow::new();
            
            let row_box = gtk4::Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .build();

            // Label
            let label = Label::new(Some(label_text));
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_hexpand(true);
            label.set_width_chars(1);
            label.set_halign(Align::Center);
            label.set_xalign(0.5);
            row_box.append(&label);
            
            row.set_child(Some(&row_box));
            
            // Store command string in data
            unsafe {
                row.set_data("command", cmd.clone());
            }

            list_box.append(&row);
        }

        launcher_box.append(&list_box);
        overlay_box.append(&launcher_box);
        window.set_child(Some(&overlay_box));

        let menu = Rc::new(RefCell::new(Self {
            window,
            list_box,
            css_provider,
        }));

        // Activation (Enter)
        let menu_clone_activate = menu.clone();
        menu.borrow().list_box.connect_row_activated(move |_, row| {
             unsafe {
                 if let Some(cmd) = row.data::<String>("command") {
                      // Execute command matches user request
                      let cmd_str = cmd.as_ref();
                      let _ = Command::new("sh")
                          .arg("-c")
                          .arg(cmd_str)
                          .spawn();
                      
                      // Close menu
                      menu_clone_activate.borrow().window.set_visible(false);
                 }
             }
        });

        // Key Navigation (Escape to close)
        let key_controller = gtk4::EventControllerKey::new();
        let menu_clone_key = menu.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                menu_clone_key.borrow().window.set_visible(false);
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        menu.borrow().window.add_controller(key_controller);

        // Click outside to close
        let click_controller = gtk4::GestureClick::new();
        let menu_clone_click = menu.clone();
        click_controller.connect_released(move |_, _, x, y| {
            let menu = menu_clone_click.borrow();
            if let Some(child) = menu.window.child() {
                if let Some(overlay) = child.first_child() {
                    let alloc = overlay.allocation();
                    let bx = alloc.x() as f64;
                    let by = alloc.y() as f64;
                    let bw = alloc.width() as f64;
                    let bh = alloc.height() as f64;
                    if x < bx || x > bx + bw || y < by || y > by + bh {
                        menu.window.set_visible(false);
                    }
                }
            }
        });
        menu.borrow().window.add_controller(click_controller);

        menu
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.window.set_visible(false);
        } else {
            // Refresh CSS from config (picks up pywal changes)
            let config = AppConfig::load().unwrap_or_default();
            self.css_provider.load_from_data(&config.generate_css());

            self.window.set_visible(true);
            self.list_box.unselect_all();
            self.list_box.grab_focus();
        }
    }
}
