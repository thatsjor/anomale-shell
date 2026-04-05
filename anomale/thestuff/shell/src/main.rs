use gtk4::prelude::*;
use gtk4::{Application, gio};
use clap::Parser;

mod bar;
mod config;
mod layout;
mod modules;
mod watcher;
mod apps;
mod power;
mod wallpapers;

use config::Config;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;
use gtk4::gdk::Monitor;
use gtk4::ApplicationWindow;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Full reload - restarts the application
    #[arg(long)]
    reload: bool,
    
    /// Quick refresh - CSS-only update without restart
    #[arg(long)]
    refresh: bool,
    
    /// Refresh CSS and run exec commands from active configs
    #[arg(long)]
    refreshexec: bool,

    /// Toggle App Launcher
    #[arg(long)]
    apps: bool,

    /// Toggle Power Menu
    #[arg(long)]
    power: bool,

    /// Toggle Wallpaper Selector
    #[arg(long)]
    wallpapers: bool,
}

fn run_command(cmd: &str) {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .ok();
}

fn spawn_restart_process() {
    // Get the path to current executable
    if let Ok(exe_path) = std::env::current_exe() {
        // Spawn new instance with delay via shell to avoid DBus race
        let exe_str = exe_path.to_string_lossy().to_string();
        println!("Spawning new instance: {}", exe_str);
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("sleep 0.5 && {}", exe_str))
            .spawn()
            .ok();
    }
}

fn restart_app(app: &Application) {
    println!("Restarting...");
    spawn_restart_process();
    // Quit current instance
    app.quit();
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location();
        let file = location.map(|l| l.file()).unwrap_or("<unknown>");
        let line = location.map(|l| l.line()).unwrap_or(0);

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        eprintln!(
            "CRASH DETECTED at {}:{}: {}",
            file,
            line,
            msg
        );
        
        // Spawn new process
        spawn_restart_process();
    }));
}

fn refresh_menu_css(provider: &gtk4::CssProvider) {
    let menus_config = config::AppConfig::load().unwrap_or_default();
    provider.load_from_data(&menus_config.generate_css());
}

fn refresh_css(
    bars: &Rc<RefCell<Vec<(Option<String>, ApplicationWindow, gtk4::CssProvider)>>>,
    menu_provider: &Option<gtk4::CssProvider>,
) {
    println!("Refreshing CSS...");
    let bars = bars.borrow();
    
    for (monitor_name, _window, provider) in bars.iter() {
        let config = Config::load(monitor_name.as_deref()).unwrap_or_else(|_| {
            Config::default()
        });
        let css = bar::generate_css(&config, monitor_name.as_deref());
        provider.load_from_data(&css);
    }

    if let Some(mp) = menu_provider {
        refresh_menu_css(mp);
    }
}

fn refresh_with_exec(
    bars: &Rc<RefCell<Vec<(Option<String>, ApplicationWindow, gtk4::CssProvider)>>>,
    menu_provider: &Option<gtk4::CssProvider>,
) {
    println!("Refreshing CSS and executing commands...");
    let bars = bars.borrow();
    let mut exec_commands = HashSet::new();
    
    for (monitor_name, _window, provider) in bars.iter() {
        let config = Config::load(monitor_name.as_deref()).unwrap_or_else(|_| {
            Config::default()
        });
        let css = bar::generate_css(&config, monitor_name.as_deref());
        provider.load_from_data(&css);
        
        for cmd in &config.exec {
            exec_commands.insert(cmd.clone());
        }
    }

    if let Some(mp) = menu_provider {
        refresh_menu_css(mp);
    }
    
    for cmd in exec_commands {
        run_command(&cmd);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_panic_hook();

    let app = Application::builder()
        .application_id("com.jor.anomale")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    let bars: Rc<RefCell<Vec<(Option<String>, ApplicationWindow, gtk4::CssProvider)>>> = Rc::new(RefCell::new(Vec::new()));
    let menu_css_provider_store: Rc<RefCell<Option<gtk4::CssProvider>>> = Rc::new(RefCell::new(None));
    
    // We need to keep track if we have initialized the bars/watcher to avoid double init
    // because activate might be called after command-line
    let is_initialized = Rc::new(RefCell::new(false));

    // Launcher state
    let app_launcher_store: Rc<RefCell<Option<Rc<RefCell<apps::AppLauncher>>>>> = Rc::new(RefCell::new(None));
    let should_launch_apps_store = Rc::new(RefCell::new(false));

    let power_menu_store: Rc<RefCell<Option<Rc<RefCell<power::PowerMenu>>>>> = Rc::new(RefCell::new(None));
    let should_show_power_store = Rc::new(RefCell::new(false));

    let wallpaper_menu_store: Rc<RefCell<Option<Rc<RefCell<wallpapers::WallpaperMenu>>>>> = Rc::new(RefCell::new(None));
    let should_show_wallpapers_store = Rc::new(RefCell::new(false));

    let bars_clone_activate = bars.clone();
    let is_initialized_clone_activate = is_initialized.clone();
    let app_launcher_store_activate = app_launcher_store.clone();
    let should_launch_apps_store_activate = should_launch_apps_store.clone();
    let power_menu_store_activate = power_menu_store.clone();
    let should_show_power_store_activate = should_show_power_store.clone();
    let wallpaper_menu_store_activate = wallpaper_menu_store.clone();
    let should_show_wallpapers_store_activate = should_show_wallpapers_store.clone();
    let menu_css_provider_store_activate = menu_css_provider_store.clone();

    app.connect_activate(move |app| {
        if *is_initialized_clone_activate.borrow() {
            return;
        }
        *is_initialized_clone_activate.borrow_mut() = true;

        // Apply last wallpaper FIRST (synchronous) so pywal colors are ready before UI loads
        {
            let menus_config = config::AppConfig::load().unwrap_or_default();
            wallpapers::apply_last_wallpaper(&menus_config);
        }

        let display = gtk4::gdk::Display::default().expect("Could not get default display");
        let monitors = display.monitors();

        let mut exec_commands = HashSet::new();
        let mut exec_once_commands = HashSet::new();

        // Debug: List config directory
        if let Ok(config_path) = Config::get_config_path(None) {
            if let Some(parent) = config_path.parent() {
                println!("DEBUG: Listing config directory: {:?}", parent);
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("DEBUG: Found file: {:?}", entry.path());
                        }
                    }
                }
            }
        }

        // Iterate over monitors and create a bar for each
        for i in 0..monitors.n_items() {
            if let Some(monitor) = monitors.item(i).and_downcast::<Monitor>() {
                if let Some(monitor_name) = monitor.connector() {
                    println!("DEBUG: Processing monitor: {}", monitor_name);
                    let config = Config::load(Some(monitor_name.as_str())).unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to load config for monitor {:?}: {}. Using defaults.", monitor_name, e);
                        Config::default()
                    });
                    println!("DEBUG: Loaded config for {} with bar_height: {}", monitor_name, config.bar_height);

                    // Collect commands
                    for cmd in &config.exec {
                        exec_commands.insert(cmd.clone());
                    }
                    for cmd in &config.exec_once {
                        exec_once_commands.insert(cmd.clone());
                    }

                     let (window, provider) = bar::create_bar(app, &monitor, &config);
                    
                    bars_clone_activate.borrow_mut().push((Some(monitor_name.into()), window, provider));
                }
            }
        }
        
        // Run startup commands
        // Deduplicated by HashSet
        for cmd in exec_once_commands {
            run_command(&cmd);
        }
        for cmd in exec_commands {
            run_command(&cmd);
        }
        
        // Setup watcher
        let (sender, receiver) = async_channel::unbounded();
        
        let mut watch_paths = Vec::new();
        // Watch config directory
        if let Ok(config_path) = Config::get_config_path(None) {
            if let Some(parent) = config_path.parent() {
                watch_paths.push(parent.to_path_buf());
            }
        }
        
        // Watch pywal colors
        if let Ok(home) = std::env::var("HOME") {
            let pywal_path = std::path::PathBuf::from(home).join(".cache/wal/colors.json");
             if let Some(parent) = pywal_path.parent() {
                 watch_paths.push(parent.to_path_buf());
             }
        }
        
        watcher::spawn_watcher(watch_paths, sender);
        
        let bars_clone = bars_clone_activate.clone();
        let menu_provider_clone = menu_css_provider_store_activate.clone();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(_) = receiver.recv().await {
                refresh_css(&bars_clone, &menu_provider_clone.borrow());
            }
        });

        // Create shared CSS provider for menus
        let menu_css_provider = gtk4::CssProvider::new();
        // Load initial CSS
        let menus_config = config::AppConfig::load().unwrap_or_default();
        menu_css_provider.load_from_data(&menus_config.generate_css());
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get default display"),
            &menu_css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER,
        );

        // Store for later refresh
        *menu_css_provider_store_activate.borrow_mut() = Some(menu_css_provider.clone());

        // Initialize Launcher
        if app_launcher_store_activate.borrow().is_none() {
             let launcher = apps::AppLauncher::new(app, &menu_css_provider);
             *app_launcher_store_activate.borrow_mut() = Some(launcher);
        }

        // Check if we need to auto-launch from command line args (first run)
        if *should_launch_apps_store_activate.borrow() {
             if let Some(launcher) = app_launcher_store_activate.borrow().as_ref() {
                 launcher.borrow().toggle();
             }
             *should_launch_apps_store_activate.borrow_mut() = false;
        }

        // Initialize Power Menu
        if power_menu_store_activate.borrow().is_none() {
             let menu = power::PowerMenu::new(app, &menu_css_provider);
             *power_menu_store_activate.borrow_mut() = Some(menu);
        }

        if *should_show_power_store_activate.borrow() {
             if let Some(menu) = power_menu_store_activate.borrow().as_ref() {
                 menu.borrow().toggle();
             }
             *should_show_power_store_activate.borrow_mut() = false;
        }

        // Initialize Wallpaper Menu
        if wallpaper_menu_store_activate.borrow().is_none() {
             let menu = wallpapers::WallpaperMenu::new(app, &menu_css_provider);
             *wallpaper_menu_store_activate.borrow_mut() = Some(menu);
        }

        if *should_show_wallpapers_store_activate.borrow() {
             if let Some(menu) = wallpaper_menu_store_activate.borrow().as_ref() {
                         wallpapers::WallpaperMenu::toggle(menu);
             }
             *should_show_wallpapers_store_activate.borrow_mut() = false;
        }

    });

    let bars_clone_cmd = bars.clone();
    let app_launcher_store_cmd = app_launcher_store.clone();
    let should_launch_apps_store_cmd = should_launch_apps_store.clone();
    let power_menu_store_cmd = power_menu_store.clone();
    let should_show_power_store_cmd = should_show_power_store.clone();
    let wallpaper_menu_store_cmd = wallpaper_menu_store.clone();
    let should_show_wallpapers_store_cmd = should_show_wallpapers_store.clone();
    let menu_css_provider_store_cmd = menu_css_provider_store.clone();
    app.connect_command_line(move |app, cmdline| {
        let args = cmdline.arguments();
        // Parse arguments directly from OsString
        println!("DEBUG: Received command line args: {:?}", args);
        match Args::try_parse_from(&args) {
            Ok(parsed) => {
                println!("DEBUG: Parsed args: {:?}", parsed);
                if parsed.reload {
                     println!("DEBUG: Reload requested");
                     // Full restart
                     restart_app(app);
                } else if parsed.refreshexec {
                     println!("DEBUG: Refresh exec requested");
                     // Refresh CSS and run exec commands
                     refresh_with_exec(&bars_clone_cmd, &menu_css_provider_store_cmd.borrow());
                } else if parsed.refresh {
                     println!("DEBUG: Refresh requested");
                     // CSS-only refresh
                     refresh_css(&bars_clone_cmd, &menu_css_provider_store_cmd.borrow());
                } else if parsed.apps {
                     println!("DEBUG: Apps toggle requested");
                     if let Some(launcher) = app_launcher_store_cmd.borrow().as_ref() {
                         launcher.borrow().toggle();
                     } else {
                          println!("DEBUG: Apps launcher not initialized yet, marking for launch");
                          *should_launch_apps_store_cmd.borrow_mut() = true;
                      }
                } else if parsed.power {
                     println!("DEBUG: Power toggle requested");
                     if let Some(menu) = power_menu_store_cmd.borrow().as_ref() {
                         menu.borrow().toggle();
                     } else {
                         println!("DEBUG: Power menu not initialized yet, marking for launch");
                         *should_show_power_store_cmd.borrow_mut() = true;
                     }
                } else if parsed.wallpapers {
                     println!("DEBUG: Wallpapers toggle requested");
                     if let Some(menu) = wallpaper_menu_store_cmd.borrow().as_ref() {
                                 wallpapers::WallpaperMenu::toggle(menu);
                     } else {
                         println!("DEBUG: Wallpaper menu not initialized yet, marking for launch");
                         *should_show_wallpapers_store_cmd.borrow_mut() = true;
                     }
                }
            }
            Err(e) => {
                eprintln!("Error parsing args: {}", e);
            }
        }


        app.activate();
        
        0 // Return status code 0
    });

    app.run();

    Ok(())
}
