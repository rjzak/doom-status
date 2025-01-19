mod assets;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::Application;

#[cfg(not(target_os = "linux"))]
mod not_linux;
#[cfg(not(target_os = "linux"))]
use not_linux::Application;

use tray_icon::menu::MenuEvent;
use tray_icon::TrayIconEvent;
use winit::event_loop::{ControlFlow, EventLoop};

pub const VERSION: &str = concat!(
    "v",
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " ",
    env!("VERGEN_BUILD_DATE")
);

#[allow(dead_code)]
#[derive(Debug)]
pub enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1].contains("version") {
        println!("doom-status {}", VERSION);
        return;
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::wait_duration(std::time::Duration::from_secs(
        2,
    )));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event)).unwrap();
    }));

    let mut app = Application::new();

    #[cfg(target_os = "linux")]
    let app_copy = app.clone();

    // Since winit doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        let local_app = app_copy;
        gtk::init().expect("failed to initialize GTK");

        local_app.init();

        gtk::main();
    });

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {:?}", err);
    }
}
