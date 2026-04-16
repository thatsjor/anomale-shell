use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, Box, Orientation, Align, Image, Revealer, RevealerTransitionType};
use gtk4_layer_shell::{Layer, LayerShell, Edge};
use crate::config::NotifyConfig;
use std::rc::Rc;

pub struct NotificationWindow {
    pub id: u32,
    pub window: ApplicationWindow,
    pub revealer: Revealer,
    pub config: NotifyConfig,
}

impl NotificationWindow {
    pub fn new(
        app: &Application,
        id: u32,
        app_name: &str,
        summary: &str,
        body: &str,
        icon_name: &str,
        config: NotifyConfig,
        monitor: Option<&gtk4::gdk::Monitor>,
    ) -> Rc<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title(format!("Notification {}", id))
            .decorated(false)
            .build();

        window.init_layer_shell();
        window.set_namespace("anomale-notification");
        window.add_css_class("anomale-notification-window");
        window.set_layer(Layer::Overlay);
        if let Some(m) = monitor {
            window.set_monitor(m);
        }

        let (v_edge, h_edge) = match config.corner.as_str() {
            "top-left" => (Edge::Top, Edge::Left),
            "top-right" => (Edge::Top, Edge::Right),
            "bottom-left" => (Edge::Bottom, Edge::Left),
            "bottom-right" => (Edge::Bottom, Edge::Right),
            _ => (Edge::Bottom, Edge::Right),
        };

        window.set_anchor(v_edge, true);
        window.set_anchor(h_edge, true);
        window.set_margin(v_edge, config.margin);
        window.set_margin(h_edge, config.margin);

        let content_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(15)
            .halign(Align::Fill)
            .valign(Align::Center)
            .hexpand(true)
            .build();
        content_box.add_css_class("notification-window");
        content_box.set_width_request(config.width);
        content_box.set_height_request(config.height);

        let icon = if !icon_name.is_empty() {
             let img = Image::from_icon_name(icon_name);
             img.set_pixel_size(48);
             img.set_valign(Align::Center);
             Some(img)
        } else {
            None
        };

        if let Some(i) = icon {
            content_box.append(&i);
        }

        let text_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .valign(Align::Center)
            .build();


        let summary_label = Label::builder()
            .label(summary)
            .halign(Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        summary_label.add_css_class("notification-summary");
        
        let body_label = Label::builder()
            .label(body)
            .halign(Align::Start)
            .use_markup(true)
            .wrap(true)
            .max_width_chars(35)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        body_label.add_css_class("notification-body");

        text_box.append(&summary_label);
        text_box.append(&body_label);
        content_box.append(&text_box);

        let revealer = Revealer::builder()
            .child(&content_box)
            .transition_type(RevealerTransitionType::Crossfade)
            .transition_duration(300)
            .hexpand(true)
            .vexpand(true)
            .build();

        window.set_child(Some(&revealer));

        Rc::new(Self {
            id,
            window,
            revealer,
            config,
        })
    }

    pub fn show(&self) {
        self.window.present();
        self.revealer.set_reveal_child(true);
    }

    pub fn hide(&self, callback: impl FnOnce() + 'static) {
        // Close the window immediately — any GTK-side fade (opacity or Revealer)
        // fades only GTK's rendered surface; the compositor's blur/shadow are applied
        // to the window region independently and linger until the window actually closes.
        // Instant close is the only way they all disappear together.
        self.window.close();
        callback();
    }

    pub fn set_y_offset(&self, offset: i32) {
        let (v_edge, _) = match self.config.corner.as_str() {
            "top-left" | "top-right" => (Edge::Top, Edge::Left),
            _ => (Edge::Bottom, Edge::Right),
        };
        self.window.set_margin(v_edge, self.config.margin + offset);
    }
}
