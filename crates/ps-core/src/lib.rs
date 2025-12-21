#[repr(packed)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl PixelRect {
    /// The "Magic Value" that signifies the end of one frame's data.
    pub const EOS_MARKER: Self = Self {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    /// Helper to check if this rect is actually the end-of-frame marker.
    #[inline]
    pub fn is_frame_end(&self) -> bool {
        self.w == 0 && self.h == 0
    }
}

pub mod file_header {
    use std::mem;

    /// The first 2 bytes of the file are the FPS (u16)
    pub const FPS_OFFSET: usize = 0;
    pub const FPS_SIZE: usize = mem::size_of::<u16>();

    /// The frame data starts immediately after the FPS
    pub const DATA_START: usize = FPS_OFFSET + FPS_SIZE;
}
