use winreg::enums::*;
use winreg::RegKey;

const REG_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "claude-view-tray";

/// Register this executable to auto-start at Windows login.
pub fn enable() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot get exe path: {e}"))?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_KEY)
        .map_err(|e| format!("Cannot open registry: {e}"))?;
    key.set_value(APP_NAME, &exe.to_string_lossy().as_ref())
        .map_err(|e| format!("Cannot set registry value: {e}"))?;
    Ok(())
}

/// Remove auto-start entry.
pub fn disable() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(REG_KEY, KEY_WRITE)
        .map_err(|e| format!("Cannot open registry: {e}"))?;
    // Ignore error if value doesn't exist
    let _ = key.delete_value(APP_NAME);
    Ok(())
}

/// Check if auto-start is enabled.
pub fn is_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey(REG_KEY) {
        Ok(k) => k,
        Err(_) => return false,
    };
    key.get_value::<String, _>(APP_NAME).is_ok()
}
