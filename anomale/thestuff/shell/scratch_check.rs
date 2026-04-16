use gtk4::prelude::*;
use gtk4::Application;

fn main() {
    let app = Application::builder().build();
    app.hold();
    app.release();
}
