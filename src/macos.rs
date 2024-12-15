use crate::common::DoomGuy;
use objc2::__framework_prelude::Retained;
use objc2::rc::Id;
use objc2_app_kit::NSMenu;
use objc2_app_kit::NSStatusBar;

pub struct MacDoomGuy {
    state: u8,
    status_bar: Retained<NSStatusBar>,
}

impl MacDoomGuy {
    pub fn init() -> Self {
        let status_bar = unsafe { NSStatusBar::systemStatusBar() };

        MacDoomGuy {
            state: 1,
            status_bar,
        }
    }
}

impl DoomGuy for MacDoomGuy {
    fn update(&self) {
        todo!()
    }

    fn run(&self) {
        //objc2_app_kit::NSMenu::new();
        //let mtm = unsafe { MainThreadMarker::new_unchecked() };
        //let menu = MenuWrapper::new(mtm);
        todo!()
    }
}
