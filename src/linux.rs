use crate::{assets, UserEvent, VERSION};

use std::sync::{Arc, RwLock};

use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tray_icon::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[derive(Clone)]
pub struct Application {
    quit_menu_item: Arc<MenuItem>,
    sys: Arc<RwLock<System>>,
    tray_icon: Arc<RwLock<Option<TrayIcon>>>,
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}

impl Application {
    pub fn new() -> Self {
        let quit_menu_item = Arc::new(MenuItem::new("Quit", true, None));

        Self {
            quit_menu_item,
            sys: Arc::new(RwLock::new(System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
            ))),
            tray_icon: Arc::new(RwLock::new(None)),
        }
    }

    pub fn init(&self) {
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
                self.quit_menu_item.as_ref(),
            ])
            .expect("Failed to create menu");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_icon(assets::load_icon(0))
            .build()
            .expect("Failed to create tray icon object");

        if let Ok(mut tray_icon_guard) = self.tray_icon.write() {
            *tray_icon_guard = Some(tray_icon);
        }
    }

    pub fn update(&self) {
        let usage = if let Ok(mut sys_guard) = self.sys.write() {
            sys_guard.refresh_cpu_all();
            sys_guard.global_cpu_usage()
        } else {
            println!("Failed to lock system cpu usage checker");
            0f32
        };

        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        if let Ok(mut tray_icon_guard) = self.tray_icon.write() {
            if let Some(tray_icon) = tray_icon_guard.as_mut() {
                tray_icon
                    .set_icon(Some(assets::load_icon(usage as u8)))
                    .expect("Failed to set icon");
            }
        }

        #[cfg(debug_assertions)]
        println!("CPU usage: {:.1}%", usage);
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
