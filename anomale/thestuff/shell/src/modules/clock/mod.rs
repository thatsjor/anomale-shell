use gtk4::prelude::*;
use gtk4::Label;
use chrono::Local;
use gtk4::glib::ControlFlow;

pub fn build(format: String) -> Label {
    let label = Label::new(None);
    label.add_css_class("clock");

    let update = {
        let label = label.clone();
        let format = format.clone();
        move || {
            let now = Local::now();
            let time_str = now.format(&format).to_string();
            label.set_text(&time_str);
            ControlFlow::Continue
        }
    };

    // Update immediately
    update();

    // Update every second
    gtk4::glib::timeout_add_seconds_local(1, update);

    label
}
