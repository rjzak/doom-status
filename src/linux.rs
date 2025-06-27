use crate::{assets, UserEvent, VERSION};

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tray_icon::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[derive(Clone)]
pub struct Application {
    quit_menu_item: Arc<Mutex<MenuItem>>,
    sys: Arc<Mutex<System>>,
    tray_icon_index: Arc<Mutex<(Option<TrayIcon>, u8)>>,
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}

impl Application {
    pub fn new() -> Self {
        // These Arc<Mutex<>> types are allowed as the other thread which uses Application
        // only performs the one-time initialization via `.init()` and nothing. GTK needs
        // its own thread and to be initialized before Application.
        #[allow(clippy::arc_with_non_send_sync)]
        let quit_menu_item = Arc::new(Mutex::new(MenuItem::new("Quit", true, None)));

        Self {
            quit_menu_item,
            sys: Arc::new(Mutex::new(System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
            ))),
            #[allow(clippy::arc_with_non_send_sync)]
            tray_icon_index: Arc::new(Mutex::new((None, 0))),
        }
    }

    pub fn init(&self) {
        let tray_menu = Menu::new();

        if let Ok(quit_menu_item_guard) = self.quit_menu_item.lock() {
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
                    quit_menu_item_guard.deref(),
                ])
                .expect("Failed to create menu");
        }

        let (image, _index) = assets::load_icon(0);
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_icon(image)
            .build()
            .expect("Failed to create tray icon object");

        if let Ok(mut tray_icon_guard) = self.tray_icon_index.lock() {
            tray_icon_guard.0 = Some(tray_icon);
        }
    }

    pub fn update(&self) {
        let usage = if let Ok(mut sys_guard) = self.sys.lock() {
            sys_guard.refresh_cpu_all();
            sys_guard.global_cpu_usage()
        } else {
            println!("Failed to lock system cpu usage checker");
            0f32
        };

        let (image, index) = assets::load_icon(usage as u8);

        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        if let Ok(mut tray_icon_guard) = self.tray_icon_index.lock() {
            if tray_icon_guard.1 != index {
                if let Some(tray_icon) = tray_icon_guard.0.as_mut() {
                    tray_icon.set_icon(Some(image)).expect("Failed to set icon");
                }
                tray_icon_guard.1 = index;
            }
        }

        #[cfg(debug_assertions)]
        println!("CPU usage: {usage:.1}%");
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            self.update();
        }

        if cause == StartCause::Poll {
            self.update();
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.update()
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        if let Ok(id) = self.quit_menu_item.lock() {
            if let UserEvent::MenuEvent(event) = event {
                if event.id == id.id() {
                    std::process::exit(0);
                }
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
