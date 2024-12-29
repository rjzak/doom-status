mod assets;

use sysinfo::System;
use tray_icon::{menu::Menu, TrayIconBuilder, TrayIconEvent};
use winit::event_loop::EventLoop;
use winit::{event::Event, event_loop::ControlFlow};

enum UserEvent {
    TrayIconEvent(TrayIconEvent),
}

fn main() {
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

    // set a tray event handler that forwards the event and wakes up the event loop
    //let proxy = event_loop.create_proxy();
    //TrayIconEvent::set_event_handler(Some(move |event| {
    //    proxy.send_event(UserEvent::TrayIconEvent(event));
    //    println!("{event:?}");
    //}));

    let mut sys = System::new_all();
    let usage = sys.global_cpu_usage();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(Menu::new()))
        .with_icon(assets::load_icon(usage as u8))
        .with_tooltip(format!("CPU usage: {:.1}%", usage))
        .build()
        .unwrap();

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
                    let icon = assets::load_icon(usage as u8);

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon.set_icon(Some(icon)).unwrap();
                    tray_icon
                        .set_tooltip(Some(format!("CPU usage: {:.1}%", usage)))
                        .unwrap();

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

                Event::AboutToWait => {
                    sys.refresh_cpu_all();
                    let usage = sys.global_cpu_usage();
                    let icon = assets::load_icon(usage as u8);

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon.set_icon(Some(icon)).unwrap();
                    tray_icon
                        .set_tooltip(Some(format!("CPU usage: {:.1}%", usage)))
                        .unwrap();
                }

                _ => {}
            }
        })
        .unwrap()
}
