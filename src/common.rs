/// Array of icons representing the state of Doom Guy
pub const ICONS: [&[u8]; 6] = [
    include_bytes!("../assets/1.png"),
    include_bytes!("../assets/2.png"),
    include_bytes!("../assets/3.png"),
    include_bytes!("../assets/4.png"),
    include_bytes!("../assets/5.png"),
    include_bytes!("../assets/6.png"),
];

/// Expected input range is a percent of overall CPU load: 0 to 100
pub fn get_icon(load: u8) -> &'static [u8] {
    match load {
        0..=16 => ICONS[0],
        17..=33 => ICONS[1],
        34..=50 => ICONS[2],
        51..=67 => ICONS[3],
        68..=84 => ICONS[4],
        _ => ICONS[5],
    }
}

/// Functions for the OS-specific DoomGuy type
pub trait DoomGuy {
    /// Check the CPU state and update the DoomGuy appearance, if needed.
    fn update(&self);

    /// OS-specific event loop.
    fn run(&self);
}
