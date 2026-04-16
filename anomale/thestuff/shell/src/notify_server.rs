use std::collections::HashMap;
use zbus::zvariant::{Value, OwnedValue};
use async_channel::Sender;

pub enum NotifyEvent {
    Notify {
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
        id_sender: tokio::sync::oneshot::Sender<u32>,
    },
    Close(u32),
    ActionInvoked(u32, String),
    NotificationClosed(u32, u32),
}

pub struct NotificationServer {
    pub events_tx: Sender<NotifyEvent>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    #[zbus(signal)]
    async fn notification_closed(ctxt: &zbus::SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(ctxt: &zbus::SignalContext<'_>, id: u32, action_key: String) -> zbus::Result<()>;

    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let (id_tx, id_rx) = tokio::sync::oneshot::channel();
        
        let mut owned_hints = HashMap::new();
        for (k, v) in hints {
            if let Ok(owned) = v.try_to_owned() {
                owned_hints.insert(k, owned);
            }
        }

        let event = NotifyEvent::Notify {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints: owned_hints,
            expire_timeout,
            id_sender: id_tx,
        };

        if let Err(e) = self.events_tx.send(event).await {
            eprintln!("Failed to send notify event: {}", e);
            return 0;
        }

        id_rx.await.unwrap_or(0)
    }

    async fn close_notification(&self, notification_id: u32) {
        let _ = self.events_tx.send(NotifyEvent::Close(notification_id)).await;
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "actions".to_string(),
            "icon-static".to_string(),
            "persistence".to_string(),
        ]
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("anomale-shell", "jor", "0.1.0", "1.2")
    }

}
