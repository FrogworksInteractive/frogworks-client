#![windows_subsystem = "windows"]

use std::process;
use single_instance::SingleInstance;
use tray_item::{IconSource, TrayItem};

fn main() {
    // Ensure that there is only a single instance of the daemon running, ever.
    let instance: SingleInstance = SingleInstance::new("frogworks_daemon").unwrap();

    if !instance.is_single() {
        process::exit(1)
    }

    // Set up the tray item.
    let mut tray_item: TrayItem = TrayItem::new(
        "Frogworks",
        IconSource::Resource("frogworks-logo")
    ).unwrap();

    tray_item.add_label("Frogworks").unwrap();

    tray_item.add_menu_item("Hello", || {
        println!("Hello!");
    }).unwrap();

    tray_item.add_menu_item("Quit", || {
        process::exit(0);
    }).unwrap();

    loop {}
}
