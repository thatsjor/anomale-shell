use gio::prelude::*;
use std::fs;
use gio::AppInfo;

fn main() {
    let desktop_file_content = r#"[Desktop Entry]
Name=Test No Exec
Type=Application
"#;
    
    fs::write("test_no_exec.desktop", desktop_file_content).unwrap();
    
    // Create DesktopAppInfo from it
    let app = gio::DesktopAppInfo::from_filename("test_no_exec.desktop").unwrap();
    
    println!("App Name: {}", app.name());
    
    // Will this panic?
    let exec = app.executable();
    println!("Exec: {:?}", exec);
}
