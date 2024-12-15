mod common;
#[cfg(target_os = "macos")]
mod macos;
use crate::common::DoomGuy;
#[cfg(target_os = "macos")]
use macos::MacDoomGuy as DoomGuyObject;

fn main() {
    let dmg = DoomGuyObject::init();
    dmg.run();
}
