use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Orientation, Align, Label};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4_layer_shell::{Layer, LayerShell, KeyboardMode};
use crate::config::AppConfig;
use std::rc::Rc;
use std::cell::RefCell;
use std::process::Command;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

/// Helper to get the command for a specific wallpaper.
/// It looks for a .txt file with the same stem as the wallpaper,
/// and uses its content as the command template if found.
/// Otherwise, it uses the default command.
fn get_wallpaper_command(wall_path_str: &str, default_cmd: &str) -> String {
    let path = PathBuf::from(wall_path_str);
    if let Some(stem) = path.file_stem() {
        if let Some(parent) = path.parent() {
            let txt_path = parent.join(stem).with_extension("txt");
            println!("DEBUG: Checking for override file at: {:?}", txt_path);
            if txt_path.exists() {
                println!("DEBUG: Override file exists!");
                if let Ok(content) = std::fs::read_to_string(&txt_path) {
                    let cmd_template = content.trim();
                    if !cmd_template.is_empty() {
                        let final_command = cmd_template.replace("[[w]]", wall_path_str);
                        println!("DEBUG: Override applied -> {}", final_command);
                        return final_command;
                    } else {
                        println!("DEBUG: Override file was empty.");
                    }
                } else {
                    println!("DEBUG: Failed to read the override file.");
                }
            } else {
                println!("DEBUG: No override file found.");
            }
        }
    }
    let final_command = default_cmd.replace("[[w]]", wall_path_str);
    println!("DEBUG: Fallback to default -> {}", final_command);
    final_command
}

/// Save the selected wallpaper path to ~/.cache/anomale/last.txt
fn save_last_wallpaper(path: &str) {
    if let Ok(home) = std::env::var("HOME") {
        let cache_dir = PathBuf::from(&home).join(".cache/anomale");
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            eprintln!("Failed to create cache dir {:?}: {}", cache_dir, e);
            return;
        }
        let last_file = cache_dir.join("last.txt");
        if let Err(e) = std::fs::write(&last_file, path) {
            eprintln!("Failed to write last wallpaper to {:?}: {}", last_file, e);
        }
    }
}

/// If use_last_wall is enabled, read last.txt and run the wallpaper command.
pub fn apply_last_wallpaper(config: &AppConfig) {
    if !config.use_last_wall {
        return;
    }
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let last_file = PathBuf::from(&home).join(".cache/anomale/last.txt");
    let wall_path = match std::fs::read_to_string(&last_file) {
        Ok(p) => p.trim().to_string(),
        Err(_) => return,
    };
    if wall_path.is_empty() {
        return;
    }
    let final_cmd = get_wallpaper_command(&wall_path, &config.wallpapers_command);
    println!("Applying last wallpaper: {}", final_cmd);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&final_cmd)
        .status();
}

pub struct WallpaperMenu {
    pub window: ApplicationWindow,
    content_box: gtk4::Box,
    css_provider: gtk4::CssProvider,
    config: AppConfig,
    frames: RefCell<Vec<gtk4::Box>>,
    paths: RefCell<Vec<String>>,
    selected: RefCell<i32>,
    cols: RefCell<i32>,
    load_version: Arc<AtomicU64>,
}

enum LoadMessage {
    Image {
        path: PathBuf,
        bytes: gtk4::glib::Bytes,
        colorspace: gtk4::gdk_pixbuf::Colorspace,
        has_alpha: bool,
        bits_per_sample: i32,
        width: i32,
        height: i32,
        rowstride: i32,
    },
    Done,
}

impl WallpaperMenu {
    pub fn new(app: &Application, css_provider_ref: &gtk4::CssProvider) -> Rc<RefCell<Self>> {
        let config = AppConfig::load().unwrap_or_else(|e| {
            eprintln!("Failed to load menus config: {}. Using defaults.", e);
            AppConfig::default()
        });

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Anomale Wallpaper Selector")
            .decorated(false)
            .visible(false)
            .build();

        window.init_layer_shell();
        window.set_namespace(&config.window_namespace);
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_exclusive_zone(-1);
        
        window.set_default_size(config.wallpapers_width, config.wallpapers_height);

        let css = config.generate_css();
        css_provider_ref.load_from_data(&css);
        let css_provider = css_provider_ref.clone();

        window.add_css_class("wallpaper-window");

        let outer_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();

        let title = Label::new(Some("Choose Your Wallpaper"));
        title.add_css_class("launcher-box");
        title.set_halign(Align::Center);
        title.set_margin_bottom(20);
        outer_box.append(&title);

        let scrolled_window = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .build();

        let content_box = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .halign(Align::Fill)
            .hexpand(true)
            .valign(Align::Start)
            .spacing(20)
            .build();

        scrolled_window.set_child(Some(&content_box));
        outer_box.append(&scrolled_window);
        window.set_child(Some(&outer_box));

        let menu = Rc::new(RefCell::new(Self {
            window,
            content_box,
            css_provider,
            config,
            frames: RefCell::new(Vec::new()),
            paths: RefCell::new(Vec::new()),
            selected: RefCell::new(-1),
            cols: RefCell::new(1),
            load_version: Arc::new(AtomicU64::new(0)),
        }));

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let menu_clone_key = menu.clone();
        
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let m = menu_clone_key.borrow();
            let total = m.frames.borrow().len() as i32;
            
            if total == 0 {
                if key == gtk4::gdk::Key::Escape {
                    m.window.set_visible(false);
                    return gtk4::glib::Propagation::Stop;
                }
                return gtk4::glib::Propagation::Proceed;
            }

            let cols = *m.cols.borrow();
            let mut sel = *m.selected.borrow();

            match key {
                k if k == gtk4::gdk::Key::Escape => {
                    m.window.set_visible(false);
                    return gtk4::glib::Propagation::Stop;
                }
                k if k == gtk4::gdk::Key::Right => {
                    sel = if sel < 0 { 0 } else { (sel + 1).min(total - 1) };
                }
                k if k == gtk4::gdk::Key::Left => {
                    sel = if sel < 0 { 0 } else { (sel - 1).max(0) };
                }
                k if k == gtk4::gdk::Key::Down => {
                    sel = if sel < 0 { 0 } else { (sel + cols).min(total - 1) };
                }
                k if k == gtk4::gdk::Key::Up => {
                    sel = if sel < 0 { 0 } else { (sel - cols).max(0) };
                }
                k if k == gtk4::gdk::Key::Return || k == gtk4::gdk::Key::KP_Enter => {
                    if sel >= 0 && sel < total {
                        let paths = m.paths.borrow();
                        let path = &paths[sel as usize];
                        save_last_wallpaper(path);
                        let final_cmd = get_wallpaper_command(path, &m.config.wallpapers_command);
                        println!("Executing: {}", final_cmd);
                        let _ = Command::new("sh")
                            .arg("-c")
                            .arg(&final_cmd)
                            .spawn();
                        m.window.set_visible(false);
                    }
                    return gtk4::glib::Propagation::Stop;
                }
                _ => return gtk4::glib::Propagation::Proceed,
            }

            m.update_selection(sel);
            
            // Re-calculate view scrolling
            let frame = &m.frames.borrow()[sel as usize];
            if let Some(parent) = m.content_box.parent() {
                if let Some(scrolled) = parent.downcast_ref::<gtk4::ScrolledWindow>() {
                    let adj = scrolled.vadjustment();
                    let alloc = frame.allocation();
                    let content_alloc = m.content_box.allocation();
                    
                    let frame_y = alloc.y() as f64 - content_alloc.y() as f64;
                    let frame_h = alloc.height() as f64;
                    
                    let view_y = adj.value();
                    let page_size = adj.page_size();
                    
                    if frame_y < view_y {
                        adj.set_value(frame_y);
                    } else if frame_y + frame_h > view_y + page_size {
                        adj.set_value(frame_y + frame_h - page_size);
                    }
                }
            }

            gtk4::glib::Propagation::Stop
        });
        
        menu.borrow().window.add_controller(key_controller);

        menu
    }

    fn update_selection(&self, new_sel: i32) {
        let frames = self.frames.borrow();
        let old_sel = *self.selected.borrow();

        if old_sel >= 0 && (old_sel as usize) < frames.len() {
            frames[old_sel as usize].remove_css_class("selected");
        }

        if new_sel >= 0 && (new_sel as usize) < frames.len() {
            let frame = &frames[new_sel as usize];
            frame.add_css_class("selected");
            frame.grab_focus();
        }

        *self.selected.borrow_mut() = new_sel;
    }

    pub fn toggle(self_rc: &Rc<RefCell<Self>>) {
        let mut m = self_rc.borrow_mut();
        if m.window.is_visible() {
            m.window.set_visible(false);
        } else {
            let config = AppConfig::load().unwrap_or_default();
            m.css_provider.load_from_data(&config.generate_css());

            m.window.set_visible(true);
            m.populate_wallpapers(self_rc);
        }
    }

    fn populate_wallpapers(&mut self, self_rc: &Rc<RefCell<Self>>) {
        println!("DEBUG: Starting progressive wallpaper load");
        
        // Bump version to cancel any previous incomplete loads
        let cur_version = self.load_version.fetch_add(1, Ordering::SeqCst) + 1;
        
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }
        self.frames.borrow_mut().clear();
        self.paths.borrow_mut().clear();
        *self.selected.borrow_mut() = -1;

        // Show scanning state
        let scan_label = Label::new(Some("Scanning directory..."));
        scan_label.set_halign(Align::Center);
        scan_label.set_margin_top(40);
        self.content_box.append(&scan_label);

        let path_str = if self.config.wallpapers_path.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                self.config.wallpapers_path.replacen("~", &home, 1)
            } else {
                self.config.wallpapers_path.clone()
            }
        } else {
            self.config.wallpapers_path.clone()
        };

        let wall_dir = PathBuf::from(&path_str);
        let thumb_size = self.config.wallpapers_thumb_size;

        if !wall_dir.exists() || !wall_dir.is_dir() {
            scan_label.set_text("Wallpapers directory does not exist or is not a directory.");
            return;
        }

        let (tx, rx) = async_channel::unbounded();

        // Background loader thread
        let version_check = self.load_version.clone();
        std::thread::spawn(move || {
            let mut entries: Vec<PathBuf> = Vec::new();
            if let Ok(dir) = std::fs::read_dir(&wall_dir) {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            let ext_lower = ext.to_string_lossy().to_lowercase();
                            if ["jpg", "jpeg", "png", "webp", "bmp", "gif", "tiff", "tif"].contains(&ext_lower.as_str()) {
                                entries.push(path);
                            }
                        }
                    }
                }
            }
            entries.sort();

            for wall_path in entries {
                if version_check.load(Ordering::SeqCst) != cur_version {
                    return; // Abort if user toggled menu again
                }

                if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&wall_path, thumb_size, thumb_size, true) {
                    let _ = tx.send_blocking(LoadMessage::Image {
                        path: wall_path,
                        bytes: pixbuf.read_pixel_bytes(),
                        colorspace: pixbuf.colorspace(),
                        has_alpha: pixbuf.has_alpha(),
                        bits_per_sample: pixbuf.bits_per_sample(),
                        width: pixbuf.width(),
                        height: pixbuf.height(),
                        rowstride: pixbuf.rowstride(),
                    });
                }
            }
            
            if version_check.load(Ordering::SeqCst) == cur_version {
                let _ = tx.send_blocking(LoadMessage::Done);
            }
        });

        // Setup progressive UI receiver
        let flow_box = gtk4::FlowBox::builder()
            .homogeneous(true)
            .row_spacing(10)
            .column_spacing(10)
            .halign(Align::Center)
            .hexpand(false)
            .valign(Align::Start)
            .selection_mode(gtk4::SelectionMode::None)
            .max_children_per_line(50)
            .min_children_per_line(1)
            .build();

        let menu_clone = Rc::downgrade(self_rc);
        let command_template = self.config.wallpapers_command.clone();
        let thumb_size_fixed = thumb_size;
        
        let version_check_rx = self.load_version.clone();
        
        gtk4::glib::idle_add_local({
            let version_check_rx_idle = version_check_rx.clone();
            let rx_idle = rx.clone();
            let menu_clone_idle = menu_clone.clone();
            let scan_label_idle = scan_label.clone();
            let flow_box_idle = flow_box.clone();
            let command_template_idle = command_template.clone();
            
            let mut first = true;
            let mut count = 0;

            move || {
                if version_check_rx_idle.load(Ordering::SeqCst) != cur_version {
                    return gtk4::glib::ControlFlow::Break;
                }

                let m_rc = match menu_clone_idle.upgrade() {
                    Some(rc) => rc,
                    None => return gtk4::glib::ControlFlow::Break,
                };

                // Process up to 3 images per frame to let GTK breathe and render
                for _ in 0..3 {
                    match rx_idle.try_recv() {
                        Ok(LoadMessage::Image { path, bytes, colorspace, has_alpha, bits_per_sample, width, height, rowstride }) => {
                            let m = m_rc.borrow();
                            let scaled = gtk4::gdk_pixbuf::Pixbuf::from_bytes(&bytes, colorspace, has_alpha, bits_per_sample, width, height, rowstride);
    
                            if first {
                                m.content_box.remove(&scan_label_idle);
                                m.content_box.append(&flow_box_idle);
                                first = false;
                            }
    
                            let texture = gtk4::gdk::Texture::for_pixbuf(&scaled);
                            let picture = gtk4::Picture::for_paintable(&texture);
                            picture.set_can_shrink(true);
                            picture.set_size_request(thumb_size_fixed, thumb_size_fixed);
    
                            let frame = gtk4::Box::builder()
                                .orientation(Orientation::Vertical)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .focusable(true)
                                .build();
                            frame.add_css_class("wallpaper-thumb");
                            frame.append(&picture);
    
                            let click = gtk4::GestureClick::new();
                            let cmd = command_template_idle.clone();
                            let path_str = path.to_string_lossy().to_string();
                            let win = m.window.clone();
                            click.connect_released(move |_, _, _, _| {
                                save_last_wallpaper(&path_str);
                                let final_cmd = get_wallpaper_command(&path_str, &cmd);
                                println!("Executing: {}", final_cmd);
                                let _ = Command::new("sh")
                                    .arg("-c")
                                    .arg(&final_cmd)
                                    .spawn();
                                win.set_visible(false);
                            });
                            frame.add_controller(click);
    
                            flow_box_idle.insert(&frame, -1);
                            m.frames.borrow_mut().push(frame);
                            m.paths.borrow_mut().push(path.to_string_lossy().to_string());
                            
                            count += 1;
                            
                            let content_width = m.content_box.width();
                            if content_width > 0 {
                                let cols = ((content_width as f64) / (thumb_size_fixed as f64 + 10.0)).floor() as i32;
                                *m.cols.borrow_mut() = cols.max(1);
                            }
                        }
                        Ok(LoadMessage::Done) => {
                            if first && count == 0 {
                                scan_label_idle.set_text("No wallpapers found.");
                            }
                            println!("DEBUG: Finished progressive wallpaper load.");
                            return gtk4::glib::ControlFlow::Break;
                        }
                        Err(async_channel::TryRecvError::Empty) => {
                            // Queue is empty, wait for next idle tick
                            break;
                        }
                        Err(async_channel::TryRecvError::Closed) => {
                            return gtk4::glib::ControlFlow::Break;
                        }
                    }
                }
                
                gtk4::glib::ControlFlow::Continue
            }
        });
    }
}
