use super::utils::is_big_endian;
use log::debug;
use std::error::Error;
use std::result::Result;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCreateKeyExW, RegGetValueW, RegSetKeyValueW, HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_WRITE, REG_DWORD,  REG_OPTION_NON_VOLATILE, REG_ROUTINE_FLAGS, RRF_RT_REG_DWORD, RRF_RT_REG_QWORD, RRF_RT_REG_SZ
};

/// Set a u32 value to the registry (as a DWORD). The key will be created if it doesn't exist.
/// 
/// * path: The registry path (includes HIVE name, eg. HKEY_LOCAL_MACHINE\....)
/// * value: The registry value name
/// 
pub fn set_u32_value(path: &str, value_name: &str, value: u32) -> Result<(), Box<dyn Error>> {
    let log_message: String = format!(
        "Setting registry value: {} -> {}: {}",
        path, value_name, value
    );
    let (hive, sub_path) = get_path_components(path)?;

    let value_wide: Vec<u16> = value_name.encode_utf16().chain(Some(0)).collect();
    let path_wide: Vec<u16> = sub_path.encode_utf16().chain(Some(0)).collect();


    // Create the key if it doesn't exist
    let mut key_handle: HKEY = HKEY::default();
    let result = unsafe {
        RegCreateKeyExW(
            hive,
            PCWSTR(path_wide.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key_handle,
            None,
        )
    };

    if result != ERROR_SUCCESS {
        debug!("{}[FAILED] (ERR = {})", log_message, result.0);
        return Err(format!("Failed to create registry key: {:?}", result).into());
    }

    let result = unsafe {
        RegSetKeyValueW(
            hive,
            PCWSTR(path_wide.as_ptr()),
            PCWSTR(value_wide.as_ptr()),
            REG_DWORD.0,
            Some(&value as *const _ as *const _),
            std::mem::size_of::<u32>() as u32,
        )
    };

    if result != ERROR_SUCCESS {
        debug!("{}[FAILED] (ERR = {})", log_message, result.0);
        return Err(format!("Failed to set registry value: {:?}", result).into());
    }

    debug!("{}[SUCCESS]", log_message);
    Ok(())
}

/// Read a value from registry
/// 
/// * path: the registry path (includes HIVE name, eg. HKEY_LOCAL_MACHINE\....)
/// * value: the registry value name
/// * datatype: the data type of the value (REG_*). Only REG_DWORD is supported for now
pub fn read_value(path: &str, value: &str, datatype: &str) -> Result<String, Box<dyn Error>> {
    let log_message: String = format!("Reading registry value: {} -> {}: ", path, value);
    let (hive, sub_path) = get_path_components(path)?;

    let value_wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let sub_path_wide: Vec<u16> = sub_path.encode_utf16().chain(Some(0)).collect();

    let mut buffer: [u16; 512] = [0; 512];
    let mut buffer_size = (buffer.len() * std::mem::size_of::<u16>()) as u32;
    let dt = get_datatype(datatype)?;
    let result = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(sub_path_wide.as_ptr()),
            PCWSTR(value_wide.as_ptr()),
            dt,
            None,
            Some(buffer.as_mut_ptr() as *mut _),
            Some(&mut buffer_size),
        )
    };

    // for integer, we assume not existing = 0, which may be a bad assumption in some cases
    if result == ERROR_FILE_NOT_FOUND {
        if dt == RRF_RT_REG_DWORD || dt == RRF_RT_REG_QWORD {
            debug!("{}[NOT FOUND] (assuming 0)", log_message);
            return Ok(0.to_string());
        }
    }
    if result != ERROR_SUCCESS {
        debug!("{}[FAILED] (ERR = {})", log_message, result.0);
        return Err(format!("Failed to read registry value: {:?}", result).into());
    }

    match dt {
        RRF_RT_REG_DWORD => {
            let bytes = [
                (buffer[0] & 0xFF) as u8,
                (buffer[0] >> 8) as u8,
                (buffer[1] & 0xFF) as u8,
                (buffer[1] >> 8) as u8,
            ];
            let value = if is_big_endian() {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            };
            debug!("{}[SUCCESS] (Value = {})", log_message, value);
            return Ok(value.to_string());
        }
        RRF_RT_REG_SZ => {
            let value_str = String::from_utf16_lossy(&buffer[..(buffer_size as usize / 2)]);
            debug!("{}[SUCCESS] (Value = {})", log_message, value_str);
            return Ok(value_str.trim_end_matches('\u{0}').to_string());
        }
        _ => return Err(format!("Unsupported registry data type {:?}", dt).into()),
    }
}

/// Split a path into hive name and subpath. E.g.
/// 
/// path: The path "HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft" will return (HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft")
/// 
fn get_path_components(path: &str) -> Result<(HKEY, String), Box<dyn Error>> {
    let mut parts = path.splitn(2, '\\');
    let hive_str = parts.next().ok_or("Invalid registry path")?;
    let reg_path = parts.next().ok_or("Invalid registry path")?;

    let hive = match hive_str {
        "HKLM" => HKEY_LOCAL_MACHINE,
        "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKCU" => HKEY_CURRENT_USER,
        "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKCR" => HKEY_CLASSES_ROOT,
        "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKU" => HKEY_USERS,
        "HKEY_USERS" => HKEY_USERS,
        "HKCC" => HKEY_CURRENT_CONFIG,
        "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        _ => return Err("Invalid registry hive".into()),
    };

    Ok((hive, reg_path.to_string()))
}

/// Map datatype from our data string to registry type. Note that it is used
/// to restrict data being read, so it's a RRF_* type not a REG_* type.
fn get_datatype(datatype: &str) -> Result<REG_ROUTINE_FLAGS, Box<dyn Error>> {
    match datatype.to_lowercase().as_str() {
        "u32" => Ok(RRF_RT_REG_DWORD),
        "i32" => Ok(RRF_RT_REG_DWORD),
        "u64" => Ok(RRF_RT_REG_QWORD),
        "i64" => Ok(RRF_RT_REG_QWORD),
        "string" => Ok(RRF_RT_REG_SZ),
        _ => Err(format!("Unsupported registry data type {}", datatype).into()),
    }
}
