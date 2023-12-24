use std::{
    ffi::OsString,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
    ptr,
};
use windows::{core::*, Win32::System::Com::*, Win32::UI::Shell::*};

pub struct DesktopWallpaper {
    interface: IDesktopWallpaper,
}

impl DesktopWallpaper {
    pub fn new() -> Result<Self> {
        com_initialized();

        let interface: IDesktopWallpaper =
            unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)? };

        Ok(Self { interface })
    }

    pub fn get(&self, index: u32) -> std::result::Result<PathBuf, String> {
        let monitor_id = unsafe {
            self.interface
                .GetMonitorDevicePathAt(index)
                .and_then(|mid| mid.to_hstring())
                .map_err(|e| e.to_string())?
        };

        let wallpaper = unsafe {
            self.interface
                .GetWallpaper(&monitor_id)
                .map_err(|e| e.to_string())?
        };

        let wallpaper_string = unsafe { OsString::from_wide(wallpaper.as_wide()) };
        let path = Path::new(&wallpaper_string);

        (path.exists() && path.is_file())
            .then_some(path.to_path_buf())
            .ok_or("Failed to get wallpaper".to_string())
    }
}

// If using winit, COM should already be initalized with COINIT_APRTMENTTHREADED.
struct ComInitialized(*mut ());

impl Drop for ComInitialized {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

thread_local! {
    static COM_INITIALIZED: ComInitialized = {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).expect("initialize COM");
            ComInitialized(ptr::null_mut())
        }
    };
}

pub fn com_initialized() {
    COM_INITIALIZED.with(|_| {});
}
