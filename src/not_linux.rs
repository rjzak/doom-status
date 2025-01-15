use crate::{assets, UserEvent, VERSION};

use std::cell::RefCell;

use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tray_icon::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub struct Application {
    quit_menu_item: MenuItem,
    sys: RefCell<System>,
    tray_icon: RefCell<TrayIcon>,
}

impl Application {
    pub fn new() -> Self {
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

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_icon(assets::load_icon(0))
            .build()
            .expect("Failed to create tray icon object");

        Self {
            quit_menu_item,
            sys: RefCell::new(System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
            )),
            tray_icon: RefCell::new(tray_icon),
        }
    }

    pub fn update(&self) {
        self.sys.borrow_mut().refresh_cpu_all();
        let usage = self.sys.borrow().global_cpu_usage();

        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        self.tray_icon
            .borrow_mut()
            .set_icon(Some(assets::load_icon(usage as u8)))
            .expect("Failed to set icon");

        #[cfg(debug_assertions)]
        println!("CPU usage: {:.1}%", usage);
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            self.update();

            // We have to request a redraw here to have the icon actually show up.
            // Winit only exposes a redraw method on the Window so we use core-foundation directly.
            #[cfg(target_os = "macos")]
            unsafe {
                use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                let rl = CFRunLoopGetMain();
                CFRunLoopWakeUp(rl);
            }
        }

        if cause == StartCause::Poll {
            self.update();
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.update()
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        if let UserEvent::MenuEvent(event) = event {
            if event.id == self.quit_menu_item.id() {
                std::process::exit(0);
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.update();
    }
}
