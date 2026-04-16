use gtk4::prelude::*;
use gtk4::Application;
use crate::config::NotifyConfig;
use crate::notification_window::NotificationWindow;
use crate::notify_server::NotifyEvent;
use std::rc::Rc;
use std::cell::RefCell;
use async_channel::Sender;

pub struct NotifyManager {
    app: Application,
    active_notifications: RefCell<Vec<Rc<NotificationWindow>>>,
    config: RefCell<NotifyConfig>,
    id_counter: std::sync::atomic::AtomicU32,
    events_tx: Sender<NotifyEvent>,
}

impl NotifyManager {
    pub fn new(app: &Application, events_tx: Sender<NotifyEvent>) -> Rc<Self> {
        let config = NotifyConfig::load().unwrap_or_default();
        
        Rc::new(Self {
            app: app.clone(),
            active_notifications: RefCell::new(Vec::new()),
            config: RefCell::new(config),
            id_counter: std::sync::atomic::AtomicU32::new(1),
            events_tx,
        })
    }

    pub fn handle_event(self: &Rc<Self>, event: NotifyEvent) {
        match event {
            NotifyEvent::Notify {
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions: _,
                hints,
                expire_timeout,
                id_sender,
            } => {
                let id = if replaces_id > 0 {
                    replaces_id
                } else {
                    self.id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                };

                // Filter out if ID already exists (replacement logic)
                self.remove_notification(id);

                let mut config = self.config.borrow().clone();
                
                // Urgency hint handling (1 = low, 2 = normal, 3 = critical)
                if let Some(urgency) = hints.get("urgency").and_then(|v| v.downcast_ref::<u8>().ok()) {
                    if urgency == 3 {
                        // Make border red for critical notifications
                        config.border_color = "#ff0000".to_string();
                    }
                }

                // Get primary monitor
                let display = gtk4::gdk::Display::default().expect("No display");
                let monitors = display.monitors();
                let monitor = if monitors.n_items() > 0 {
                    monitors.item(0).and_downcast::<gtk4::gdk::Monitor>()
                } else {
                    None
                };

                let notify_win = NotificationWindow::new(
                    &self.app,
                    id,
                    &app_name,
                    &summary,
                    &body,
                    &app_icon,
                    config,
                    monitor.as_ref(),
                );

                // Add to active
                self.active_notifications.borrow_mut().insert(0, notify_win.clone());
                
                // Apply CSS globally to the display so it reaches all notification widgets
                let css_provider = gtk4::CssProvider::new();
                css_provider.load_from_data(&self.config.borrow().generate_css());
                if let Some(display) = gtk4::gdk::Display::default() {
                    gtk4::style_context_add_provider_for_display(
                        &display,
                        &css_provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_USER,
                    );
                }

                // Setup interactions
                let gesture = gtk4::GestureClick::new();
                let id_clone = id;
                let tx_clone = self.events_tx.clone();
                gesture.connect_pressed(move |_, _, _, _| {
                    let _ = tx_clone.send_blocking(NotifyEvent::ActionInvoked(id_clone, "default".to_string()));
                });
                notify_win.window.add_controller(gesture);

                // Set timeout for removal
                let manager_clone = self.clone();
                let timeout_ms = if expire_timeout > 0 {
                    expire_timeout as u32
                } else {
                    (self.config.borrow().timeout * 1000) as u32
                };

                gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(timeout_ms as u64), move || {
                    manager_clone.dismiss_notification(id);
                });

                // Update all positions
                self.update_positions();

                // Show
                notify_win.show();

                let _ = id_sender.send(id);
            }
            NotifyEvent::Close(id) => {
                self.dismiss_notification(id);
            }
            _ => {}
        }
    }

    fn remove_notification(&self, id: u32) {
        let mut active = self.active_notifications.borrow_mut();
        if let Some(pos) = active.iter().position(|n| n.id == id) {
            let n = active.remove(pos);
            n.window.close();
        }
    }

    fn dismiss_notification(self: &Rc<Self>, id: u32) {
        let n: Option<Rc<NotificationWindow>> = {
            let active = self.active_notifications.borrow();
            active.iter().find(|n| n.id == id).cloned()
        };

        if let Some(n) = n {
            let manager_clone = self.clone();
            let tx_clone = self.events_tx.clone();
            n.hide(move || {
                let mut active = manager_clone.active_notifications.borrow_mut();
                if let Some(pos) = active.iter().position(|anim_n| anim_n.id == id) {
                    active.remove(pos);
                }
                drop(active); // Drop borrow before calling update_positions
                manager_clone.update_positions();
                let _ = tx_clone.send_blocking(NotifyEvent::NotificationClosed(id, 2)); // 2 = expired
            });
        }
    }

    fn update_positions(&self) {
        let active = self.active_notifications.borrow();
        let config = self.config.borrow();
        let mut current_offset = 0;

        for n in active.iter() {
            n.set_y_offset(current_offset);
            current_offset += config.height + config.spacing;
        }
    }
}
