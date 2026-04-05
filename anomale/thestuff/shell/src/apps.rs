use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, ListBox, ListBoxRow, Entry, ScrolledWindow, Orientation, Align, SelectionMode, Image};
use gtk4_layer_shell::{Layer, LayerShell, Edge, KeyboardMode};
use gtk4::gio;
use crate::config::AppConfig;
use std::rc::Rc;
use std::cell::RefCell;

pub struct AppLauncher {
    pub window: ApplicationWindow,
    pub search_entry: Entry,
    pub list_box: ListBox,
    pub scrolled_window: ScrolledWindow,
    pub apps: RefCell<Vec<gio::AppInfo>>,
    pub current_matches: RefCell<Vec<gio::AppInfo>>,
    css_provider: gtk4::CssProvider,
}

impl AppLauncher {
    pub fn new(app: &Application, css_provider_ref: &gtk4::CssProvider) -> Rc<RefCell<Self>> {
        let config = AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load apps config: {}. Using defaults.", e);
            AppConfig::default()
        });

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Anomale Launcher")
            .decorated(false)
            .visible(false) // Initially hidden
            .build();

        // Layer Shell Setup - Full screen overlay
        window.init_layer_shell();
        window.set_namespace("anomale-appmenu");
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_exclusive_zone(-1); // Cover everything including the bar
        
        // Anchor all edges for full-screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Apply CSS (Initial load)
        let css = config.generate_css(None);
        css_provider_ref.load_from_data(&css);
        let css_provider = css_provider_ref.clone();

        // Full-screen overlay container - positions launcher content at top-center
        let overlay_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(Align::Center)
            .valign(Align::Fill)
            .margin_top(200)
            .vexpand(true)
            .build();
        window.add_css_class("fullscreen-bg");

        // Inner launcher box with border/styling
        let launcher_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .vexpand(true)
            .build();
        launcher_box.set_width_request(config.search_width + 10);
        launcher_box.add_css_class("launcher-box");

        // Results List
        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();
        list_box.add_css_class("app-list");

        // Search Entry
        let search_entry = Entry::builder()
            .placeholder_text("Search apps...")
            .build();
        search_entry.add_css_class("search-entry");
        
        // Search Entry Key Controller
        let entry_controller = gtk4::EventControllerKey::new();
        let list_box_clone_entry = list_box.clone();
        entry_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Down {
                if let Some(first) = list_box_clone_entry.row_at_index(0) {
                    list_box_clone_entry.select_row(Some(&first));
                    first.grab_focus();
                }
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        search_entry.add_controller(entry_controller);

        launcher_box.append(&search_entry);
        
        // List Box Key Controller (for Up navigation back to entry)
        let list_controller = gtk4::EventControllerKey::new();
        let search_entry_clone_list = search_entry.clone();
        list_controller.connect_key_pressed(move |controller, key, _, _| {
             if key == gtk4::gdk::Key::Up {
                 if let Some(widget) = controller.widget() {
                    if let Some(list_box_widget) = widget.downcast_ref::<ListBox>() {
                        if let Some(row) = list_box_widget.selected_row() {
                            if row.index() == 0 {
                                    search_entry_clone_list.grab_focus();
                                    return gtk4::glib::Propagation::Stop;
                            }
                        }
                    }
                 }
             }
             gtk4::glib::Propagation::Proceed
        });
        list_box.add_controller(list_controller);



        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_box)
            .vexpand(true)
            .build();

        // Auto-scroll to selected row
        let scrolled_window_clone_nav = scrolled_window.clone();
        list_box.connect_row_selected(move |_, row| {
             if let Some(row) = row {
                 let adj = scrolled_window_clone_nav.vadjustment();
                 let allocation = row.allocation();
                 let y = allocation.y() as f64;
                 let height = allocation.height() as f64;
                 let val = adj.value();
                 let page = adj.page_size();
                 
                 if y < val {
                     adj.set_value(y);
                 } else if y + height > val + page {
                     adj.set_value(y + height - page);
                 }
             }
        });

        scrolled_window.set_visible(false);

        launcher_box.append(&scrolled_window);
        overlay_box.append(&launcher_box);
        window.set_child(Some(&overlay_box));

        // Load Apps
        let apps = gio::AppInfo::all();
        
        let list_box_clone_enter = list_box.clone();

        let launcher = Rc::new(RefCell::new(Self {
            window,
            search_entry,
            list_box,
            scrolled_window,
            apps: RefCell::new(apps),
            current_matches: RefCell::new(Vec::new()),
            css_provider,
        }));

        // Signals
        let launcher_clone = launcher.clone();
        launcher.borrow().search_entry.connect_changed(move |entry| {
            let text = entry.text();
            let launcher = launcher_clone.borrow();
            launcher.filter_apps(&text);
        });
        
        // Enter key in Entry launches selected app
        let launcher_clone_enter = launcher.clone();
        launcher.borrow().search_entry.connect_activate(move |_| {
            // If a row is selected in the listbox, launch it
            let app_to_launch = if let Some(row) = list_box_clone_enter.selected_row() {
                 let idx = row.index();
                 if idx >= 0 {
                     let apps_ref = launcher_clone_enter.borrow();
                     let app = apps_ref.current_matches.borrow().get(idx as usize).cloned();
                     app
                 } else {
                     None
                 }
            } else {
                 // If no row is selected, but results exist, launch the first one (index 0)
                  let apps_ref = launcher_clone_enter.borrow();
                  let app = apps_ref.current_matches.borrow().first().cloned();
                  app
            };

            if let Some(app) = app_to_launch {
                let ctx = gtk4::gdk::Display::default()
                      .map(|d| d.app_launch_context());
                let launch_ctx: Option<&gio::AppLaunchContext> = ctx.as_ref().map(|c| c.upcast_ref());
                if let Err(e) = app.launch(&[], launch_ctx) {
                      eprintln!("Failed to launch app: {}", e);
                } else {
                      launcher_clone_enter.borrow().window.set_visible(false);
                }
            }
        });
        
        let launcher_clone_activate = launcher.clone();
        launcher.borrow().list_box.connect_row_activated(move |_, row| {
             let idx = row.index();
             if idx >= 0 {
                 let apps_ref = launcher_clone_activate.borrow();
                 let app_to_launch = apps_ref.current_matches.borrow().get(idx as usize).cloned();
                 // Drop the borrow of apps_ref before launching (though less critical here as we aren't using it in the if body fundamentally, but good practice)
                 drop(apps_ref);

                 if let Some(app) = app_to_launch {
                      let ctx = gtk4::gdk::Display::default()
                          .map(|d| d.app_launch_context());
                      let launch_ctx: Option<&gio::AppLaunchContext> = ctx.as_ref().map(|c| c.upcast_ref());
                      if let Err(e) = app.launch(&[], launch_ctx) {
                          eprintln!("Failed to launch app: {}", e);
                      } else {
                          launcher_clone_activate.borrow().window.set_visible(false);
                      }
                 }
             }
        });

        // Handle Escape to close
        let key_controller = gtk4::EventControllerKey::new();
        let launcher_clone_key = launcher.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                launcher_clone_key.borrow().window.set_visible(false);
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        launcher.borrow().window.add_controller(key_controller);

        // Click outside the launcher box to close
        let click_controller = gtk4::GestureClick::new();
        let launcher_clone_click = launcher.clone();
        click_controller.connect_released(move |_, _, x, y| {
            let launcher = launcher_clone_click.borrow();
            // Check if click is outside the launcher box area
            // The overlay_box centers the launcher_box, so we check bounds
            if let Some(child) = launcher.window.child() {
                if let Some(overlay) = child.first_child() {
                    let alloc = overlay.allocation();
                    let bx = alloc.x() as f64;
                    let by = alloc.y() as f64;
                    let bw = alloc.width() as f64;
                    let bh = alloc.height() as f64;
                    if x < bx || x > bx + bw || y < by || y > by + bh {
                        launcher.window.set_visible(false);
                    }
                }
            }
        });
        launcher.borrow().window.add_controller(click_controller);

        launcher
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.window.set_visible(false);
        } else {
            // Refresh CSS from config (picks up pywal changes)
            let config = AppConfig::load().unwrap_or_default();
            self.css_provider.load_from_data(&config.generate_css(None));

            // Refresh apps list
            *self.apps.borrow_mut() = gio::AppInfo::all();
            
            self.window.set_visible(true);
            self.search_entry.set_text("");
            self.search_entry.grab_focus();
            self.search_entry.remove_css_class("has-results");
            self.scrolled_window.set_visible(false);
            self.scrolled_window.set_size_request(-1, 0);
        }
    }


    fn filter_apps(&self, query: &str) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Clear matches
        self.current_matches.borrow_mut().clear();

        if query.trim().is_empty() {
            self.scrolled_window.set_visible(false);
            self.scrolled_window.set_size_request(-1, 0);
            self.search_entry.remove_css_class("has-results");
            return;
        }

        self.scrolled_window.set_visible(true);

        let query_lower = query.to_lowercase();
        
        // Scoring logic
        let apps = self.apps.borrow();
        let mut matches: Vec<(i32, gio::AppInfo)> = apps.iter().filter_map(|app| {
            let name = app.name().to_string();
            let name_lower = name.to_lowercase();
            let display_name = app.display_name().to_string();
            let display_name_lower = display_name.to_lowercase();
            let mut score = 0;

            if name_lower == query_lower || display_name_lower == query_lower {
                score = 100;
            } else if name_lower.starts_with(&query_lower) || display_name_lower.starts_with(&query_lower) {
                score = 80;
            } else if name_lower.contains(&query_lower) || display_name_lower.contains(&query_lower) {
                score = 60;
            } else {
                // Check executable (commandline) without panicking
                if let Some(cmd) = app.commandline() {
                    let cmd_lower = cmd.to_string_lossy().to_lowercase();
                    if !cmd_lower.is_empty() && cmd_lower.contains(&query_lower) {
                         score = 40;
                    }
                }
            }

            if score > 0 {
                Some((score, app.clone()))
            } else {
                None
            }
        }).collect();

        // Sort by score desc, then name asc
        matches.sort_by(|a, b| {
            b.0.cmp(&a.0).then_with(|| a.1.name().cmp(&b.1.name()))
        });

        // Deduplicate by display name (keep highest-scored entry)
        let mut seen_names = std::collections::HashSet::new();
        matches.retain(|(_, app)| seen_names.insert(app.name().to_string()));
        
        if matches.is_empty() {
             self.search_entry.remove_css_class("has-results");
             self.scrolled_window.set_visible(false);
             self.scrolled_window.set_size_request(-1, 0);
             return;
        }
        
        self.search_entry.add_css_class("has-results");

        // Take top 50, store them in matches, and create rows
        let mut row_matches = Vec::new();
        for (_, app) in matches.into_iter().take(50) {
            row_matches.push(app.clone());

            let row = ListBoxRow::new();
            
            // Horizontal box for icon + label
            let row_box = gtk4::Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .build();

            // App icon
            let icon_widget = if let Some(icon) = app.icon() {
                let img = Image::from_gicon(&icon);
                img.set_pixel_size(24);
                img
            } else {
                let img = Image::from_icon_name("application-x-executable");
                img.set_pixel_size(24);
                img
            };
            icon_widget.add_css_class("app-icon");
            row_box.append(&icon_widget);

            // App name
            let label = Label::new(Some(&app.name()));
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_hexpand(true);
            label.set_width_chars(1); // Allow shrinking below natural width
            label.set_xalign(0.0);
            row_box.append(&label);
            
            row.set_child(Some(&row_box));
            
            // NO unsafe set_data!

            self.list_box.append(&row);
        }

        // Update the stored matches to sync with the ListBox
        *self.current_matches.borrow_mut() = row_matches;
        
        // Let scrolled window expand to fill remaining space
        self.scrolled_window.set_size_request(-1, -1);

        // Auto-select first row if matches found
        if let Some(row) = self.list_box.first_child() {
             if let Some(row_widget) = row.downcast_ref::<ListBoxRow>() {
                  self.list_box.select_row(Some(row_widget));
             }
        }
    }
}

