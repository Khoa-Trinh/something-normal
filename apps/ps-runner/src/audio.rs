use windows::Win32::{
    Media::Audio::{
        eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
    },
    System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
};

/// Handles the COM lifecycle for the audio thread.
/// When this struct drops (at the end of main), it calls CoUninitialize automatically.
pub struct AudioSystem {}

impl AudioSystem {
    pub fn new() -> Self {
        unsafe {
            // Initialize COM on this thread (STA)
            let _ = CoInitialize(None);
        }
        Self {}
    }

    /// Tries to get the system volume control interface
    pub fn get_volume_control(&self) -> Option<IAudioEndpointVolume> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
            device.Activate(CLSCTX_ALL, None).ok()
        }
    }
}

// Automatic cleanup when the program exits
impl Drop for AudioSystem {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
