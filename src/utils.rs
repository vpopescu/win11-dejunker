use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::Security::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation, GetTokenInformation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Test if running on big endian
pub fn is_big_endian() -> bool {
    cfg!(target_endian = "big")
}


/// test if running elevated
pub fn is_elevated() -> bool {
    unsafe {
        let mut token_handle: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut return_length = 0;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );
            _ = CloseHandle(token_handle);
            
            if result.is_ok() {
                return elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}
