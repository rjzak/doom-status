/// Array of icons representing the state of Doom Guy
pub const ICONS: [&[u8]; 6] = [
    include_bytes!("../assets/1.png"),
    include_bytes!("../assets/2.png"),
    include_bytes!("../assets/3.png"),
    include_bytes!("../assets/4.png"),
    include_bytes!("../assets/5.png"),
    include_bytes!("../assets/6.png"),
];

pub fn get_icon(load: f32) -> &'static [u8] {
    if load >= 0.98 {
        return ICONS[ICONS.len() - 1];
    }
    const STEPS: f32 = 100.0 / ICONS.len() as f32;
    let mut step = 0.0;
    let mut index = 0usize;
    while load < step {
        step += STEPS;
        index += 1;
    }
    ICONS[index]
}

/// Functions for the OS-specific DoomGuy type
pub trait DoomGuy {
    /// Check the CPU state and update the DoomGuy appearance, if needed.
    fn update(&self);

    /// OS-specific event loop.
    fn run(&self);
}
