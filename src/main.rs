// TOTP Windows Desktop Application
// Minimalistic TOTP authenticator with native Windows UI

#![windows_subsystem = "windows"]

mod file;
mod totp;
mod ui;
mod clipboard;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    unsafe {
        // Get application instance
        let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);
        
        let (accounts, invalid) = match file::load_secrets("secrets.txt") {
            Ok(r) => r,
            Err(e) => {
                let msg = HSTRING::from(format!("Failed to load secrets.txt:\n{e}"));
                MessageBoxW(None, PCWSTR(msg.as_ptr()), w!("Error"), MB_OK | MB_ICONERROR);
                return Ok(());
            }
        };
        ui::init_window_state(accounts, invalid);
        let _hwnd = ui::create_window(hinstance)?;
        
        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        
        Ok(())
    }
}

