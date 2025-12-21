use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;

// A "Magic" string to identify if the EXE has been patched
const MAGIC: &[u8; 8] = b"PS_PATCH";

pub struct LoadedAssets {
    pub video_data: Vec<u8>,
    pub audio_data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

// This struct must match EXACTLY what the CLI writes to the end of the file
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PayloadFooter {
    video_offset: u64,
    video_len: u64,
    audio_offset: u64,
    audio_len: u64,
    width: u16,
    height: u16,
    magic: [u8; 8],
}

pub fn load() -> LoadedAssets {
    // 1. Open the current running executable
    let current_exe = env::current_exe().expect("Failed to get exe path");
    let mut file = File::open(current_exe).expect("Failed to open self");

    // 2. Seek to the end to find the footer
    let footer_size = mem::size_of::<PayloadFooter>() as i64;
    file.seek(SeekFrom::End(-footer_size)).expect("Seek failed");

    // 3. Read the footer
    let mut footer_buffer = vec![0u8; footer_size as usize];
    file.read_exact(&mut footer_buffer)
        .expect("Failed to read footer");

    // 4. Parse raw bytes back into the Struct
    // unsafe is needed to cast raw bytes to a struct
    let footer: PayloadFooter = unsafe { std::ptr::read(footer_buffer.as_ptr() as *const _) };

    // 5. Verify Magic
    if &footer.magic != MAGIC {
        // Fallback for Development: If you run 'cargo run', it won't be patched.
        // You might want to return dummy data or panic here.
        panic!("❌ FATAL: This runner is a template. It has not been patched with assets.");
    }

    // 6. Read Video
    let mut video_data = vec![0u8; footer.video_len as usize];
    file.seek(SeekFrom::Start(footer.video_offset))
        .expect("Seek video failed");
    file.read_exact(&mut video_data).expect("Read video failed");

    // 7. Read Audio
    let mut audio_data = vec![0u8; footer.audio_len as usize];
    file.seek(SeekFrom::Start(footer.audio_offset))
        .expect("Seek audio failed");
    file.read_exact(&mut audio_data).expect("Read audio failed");

    LoadedAssets {
        video_data,
        audio_data,
        width: footer.width,
        height: footer.height,
    }
}
