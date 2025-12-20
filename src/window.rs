#[cfg(feature = "res_720p")]
pub const BASE_WIDTH: u16 = 1280;
#[cfg(feature = "res_720p")]
pub const BASE_HEIGHT: u16 = 720;

#[cfg(feature = "res_1080p")]
pub const BASE_WIDTH: u16 = 1920;
#[cfg(feature = "res_1080p")]
pub const BASE_HEIGHT: u16 = 1080;

#[cfg(feature = "res_1440p")]
pub const BASE_WIDTH: u16 = 2560;
#[cfg(feature = "res_1440p")]
pub const BASE_HEIGHT: u16 = 1440;

#[cfg(feature = "res_2160p")]
pub const BASE_WIDTH: u16 = 3840;
#[cfg(feature = "res_2160p")]
pub const BASE_HEIGHT: u16 = 2160;

#[cfg(not(any(feature = "res_1080p", feature = "res_1440p", feature = "res_2160p")))]
pub const BASE_WIDTH: u16 = 1920;
#[cfg(not(any(feature = "res_1080p", feature = "res_1440p", feature = "res_2160p")))]
pub const BASE_HEIGHT: u16 = 1080;

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct RawWinCoords {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}
