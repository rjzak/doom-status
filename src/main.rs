mod assets;

use std::sync::{Arc, RwLock};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tray_icon::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

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
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

#[derive(Clone)]
struct Application {
    quit_menu_item: Arc<MenuItem>,
    sys: Arc<RwLock<System>>,
    tray_icon: Arc<RwLock<Option<TrayIcon>>>,
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}

impl Application {
    fn new() -> Self {
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
            #[cfg(not(target_os = "linux"))]
            self.init();

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
        gtk::init().unwrap();

        local_app.init();

        gtk::main();
    });

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {:?}", err);
    }
}
