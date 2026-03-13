// Clipboard operations module
// Win32 clipboard API for copying text

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::DataExchange::*,
    Win32::System::Memory::*,
};

/// Copy text to clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    unsafe {
        // Open clipboard
        OpenClipboard(None)?;
        
        // Clear existing clipboard content
        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(e);
        }
        
        // Convert text to wide string (UTF-16)
        let wide_text: Vec<u16> = text.encode_utf16()
            .chain(std::iter::once(0)) // null terminator
            .collect();
        
        // Allocate global memory
        let size = wide_text.len() * std::mem::size_of::<u16>();
        let hglob = GlobalAlloc(GMEM_MOVEABLE, size)?;
        
        if hglob.is_invalid() {
            let _ = CloseClipboard();
            return Err(Error::from(E_OUTOFMEMORY));
        }
        
        // Lock memory and copy text
        let locked = GlobalLock(hglob);
        if locked.is_null() {
            let _ = GlobalFree(hglob);
            let _ = CloseClipboard();
            return Err(Error::from_win32());
        }
        
        std::ptr::copy_nonoverlapping(
            wide_text.as_ptr(),
            locked as *mut u16,
            wide_text.len(),
        );
        
        let _ = GlobalUnlock(hglob);
        
        // Set clipboard data - convert HGLOBAL to HANDLE
        SetClipboardData(13, HANDLE(hglob.0 as isize))?; // 13 = CF_UNICODETEXT
        
        // Close clipboard
        let _ = CloseClipboard();
        
        Ok(())
    }
}
