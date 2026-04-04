use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;

pub fn spawn_watcher(paths: Vec<PathBuf>, sender: async_channel::Sender<()>) {
    thread::spawn(move || {
        let (tx, rx) = channel();
        
        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create watcher: {:?}", e);
                return;
            }
        };

        for path in &paths {
            if let Err(_e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                // validation or logging could go here
            } else {
                // watching started
            }
        }
        
        // Keep the watcher alive by looping on the receiver
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        Ok(event) => {
                             
                             // Basic debouncing
                             // We are interested in Modify and Create (and maybe Remove? if file is replaced)
                             
                             // Filter for interesting files if watching directories
                             let mut interesting = false;
                             for path_buf in &event.paths {
                                 if let Some(name) = path_buf.file_name().and_then(|n| n.to_str()) {
                                     if name == "colors.json" || name.ends_with(".conf") || name == "style.css" {
                                         interesting = true;
                                         break;
                                     }
                                 }
                             }
                             
                             if !interesting {
                                 continue;
                             }
                             
                             // Debounce: Wait 200ms for more events to settle
                             thread::sleep(Duration::from_millis(200));
                             
                             // Drain channel
                             while let Ok(_) = rx.try_recv() {}
                             
                             // Send update signal
                             if sender.send_blocking(()) .is_err() {
                                 break;
                             }
                        },
                        Err(_e) => {},
                    }
                },
                Err(_e) => {
                    break;
                }
            }
        }
    });
}
