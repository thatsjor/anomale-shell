use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, Align};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use async_channel;

#[derive(Debug, Clone, Copy)]
struct TagState {
    selected: bool,
    occupied: bool,
    urgent: bool,
}

pub fn build(monitor: &gtk4::gdk::Monitor) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    container.add_css_class("tags-container");

    let monitor_name = monitor.connector().unwrap_or_else(|| "Unknown".into());
    let buttons = Arc::new(Mutex::new(HashMap::new()));
    
    
    // Create 9 tag buttons initially (1-9)
    for i in 1..=9 {
        let button = Button::builder()
            .has_frame(false)
            .build();
        button.add_css_class("tag");

        // Inner layout: Dot + Number
        let bbox = Box::new(Orientation::Horizontal, 2); // spacing 2px
        let dot = Label::new(Some("●"));
        dot.add_css_class("dot");
        dot.set_valign(Align::Center);
        
        let num = Label::new(Some(&i.to_string()));
        num.add_css_class("num");
        num.set_valign(Align::Center);

        bbox.append(&dot);
        bbox.append(&num);
        button.set_child(Some(&bbox));
        
        let tag_id = i;
        button.connect_clicked(move |_| {
            let _ = Command::new("mmsg")
                .arg("-t")
                .arg(tag_id.to_string())
                .spawn();
        });

        container.append(&button);
        buttons.lock().unwrap().insert(i, button);
    }

    // Use async-channel
    let (sender, receiver) = async_channel::unbounded();
        
    let monitor_name_clone = monitor_name.clone();

    // Spawn mmsg watcher
    thread::spawn(move || {
        let child = Command::new("mmsg")
            .arg("-w")
            .arg("-t")
            .stdout(Stdio::piped())
            .spawn();

        if let Ok(mut child) = child {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // Expected format: <monitor> tag <id> <selected> <occupied> <urgent>
                        // e.g., "DP-2 tag 1 1 3 1"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 6 && parts[0] == monitor_name_clone && parts[1] == "tag" {
                            if let (Ok(id), Ok(state_val), Ok(clients_cnt)) = (
                                parts[2].parse::<i32>(),
                                parts[3].parse::<i32>(),
                                parts[4].parse::<i32>(),
                            ) {
                                let state = TagState {
                                    selected: state_val == 1, 
                                    occupied: clients_cnt > 0, 
                                    urgent: state_val == 2,
                                };
                                let _ = sender.send_blocking((id, state));
                            }
                        }
                    }
                }
            }
        }
    });

    // Handle updates in main thread
    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Ok((id, state)) = receiver.recv().await {
            if let Ok(buttons) = buttons.lock() {
                if let Some(button) = buttons.get(&id) {
                    if state.selected {
                        button.add_css_class("selected");
                    } else {
                        button.remove_css_class("selected");
                    }

                    if state.occupied {
                        button.add_css_class("occupied");
                    } else {
                        button.remove_css_class("occupied");
                    }

                    if state.urgent {
                        button.add_css_class("urgent");
                    } else {
                        button.remove_css_class("urgent");
                    }
                    
                    if !state.occupied && !state.selected {
                         button.set_visible(false);
                    } else {
                         button.set_visible(true);
                    }
                }
            }
        }
    });
    
    // Trigger initial state
    let _ = Command::new("mmsg").arg("-g").arg("-t").spawn();

    container
}
