mod assets;
use assets::load_icon;

use sysinfo::System;
use tray_icon::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};
use winit::event_loop::EventLoop;
use winit::{event::Event, event_loop::ControlFlow};

pub const VERSION: &str = concat!(
    "v",
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " ",
    env!("VERGEN_BUILD_DATE")
);

const FAILED_TO_SET_ICON: &str = "Failed to set icon";

#[cfg(debug_assertions)]
const FAILED_TO_SET_TOOLTIP: &str = "Failed to set utilization tooltip";

#[allow(dead_code)]
#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1].contains("version") {
        println!("doom-status {}", VERSION);
        return;
    }

    // Since winit doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        use tray_icon::menu::Menu;

        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        let usage = sys.global_cpu_usage();
        let icon = assets::load_icon(usage as u8);

        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_icon(icon)
            .with_tooltip(format!("CPU usage: {:.1}%", usage))
            .build()
            .unwrap();

        gtk::main();
    });

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy.send_event(UserEvent::MenuEvent(event)).unwrap();
    }));

    let quit_menu_item = MenuItem::new("Quit", true, None);
    let tray_menu = Menu::new();
    tray_menu
        .append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("doom-status".to_string()),
                    copyright: Some("Copyright rjzak".to_string()),
                    version: Some(VERSION.to_string()),
                    short_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    icon: Some(assets::icon_zero_muda()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_menu_item,
        ])
        .expect("Failed to create menu");

    let mut sys = System::new_all();
    let usage = sys.global_cpu_usage();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(load_icon(usage as u8))
        .with_tooltip(format!("CPU usage: {:.1}%", usage))
        .build()
        .expect("Failed to create tray icon object");

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::wait_duration(
                std::time::Duration::from_secs(2),
            ));

            match event {
                #[cfg(not(target_os = "linux"))]
                Event::NewEvents(winit::event::StartCause::Init) => {
                    sys.refresh_cpu_all();
                    let usage = sys.global_cpu_usage();

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon
                        .set_icon(Some(load_icon(usage as u8)))
                        .expect(FAILED_TO_SET_ICON);

                    // This causes flashing on some platforms
                    #[cfg(debug_assertions)]
                    tray_icon
                        .set_tooltip(Some(format!("CPU usage: {:.1}%", usage)))
                        .expect(FAILED_TO_SET_TOOLTIP);

                    // We have to request a redraw here to have the icon actually show up.
                    // Winit only exposes a redraw method on the Window so we use core-foundation directly.
                    #[cfg(target_os = "macos")]
                    unsafe {
                        use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                        let rl = CFRunLoopGetMain();
                        CFRunLoopWakeUp(rl);
                    }
                }

                Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
                    println!("{event:?}");
                }

                Event::UserEvent(UserEvent::MenuEvent(event)) => {
                    if event.id == quit_menu_item.id() {
                        std::process::exit(0);
                    }
                }

                Event::AboutToWait => {
                    sys.refresh_cpu_all();
                    let usage = sys.global_cpu_usage();

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon
                        .set_icon(Some(load_icon(usage as u8)))
                        .expect(FAILED_TO_SET_ICON);

                    // This causes flashing on some platforms
                    #[cfg(debug_assertions)]
                    tray_icon
                        .set_tooltip(Some(format!("CPU usage: {:.1}%", usage)))
                        .expect(FAILED_TO_SET_TOOLTIP);
                }

                _ => {}
            }
        })
        .unwrap()
}
