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
#[inline]
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

/// Converts the build-in PNG to [tray_icon::Icon]
#[inline]
pub fn load_icon(load: u8) -> tray_icon::Icon {
    let bytes = get_icon(load);
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes)
            .expect("Failed to parse icon bytes")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to parse icon")
}

/// Using [muda::Icon] here directly since there seems to be an import error with `tray_icon`
#[inline]
pub fn icon_zero_muda() -> muda::Icon {
    let bytes = get_icon(0);
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes)
            .expect("Failed to parse icon bytes")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    muda::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to parse icon")
}
