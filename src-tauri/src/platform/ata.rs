use crate::error::{Result, SecDelError};
use std::mem;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
const IOCTL_ATA_PASS_THROUGH: u32 = 0x0004D040;

#[derive(Debug, Clone)]
pub struct AtaIdentifyData {
    pub secure_erase_supported: bool,
    pub enhanced_erase_supported: bool,
    pub is_frozen: bool,
    pub is_ssd: bool,
    pub model: String,
    pub firmware: String,
    pub serial: String,
}

#[repr(C, packed)]
struct AtaPassThroughEx {
    length: u16,
    ata_flags: u16,
    data_transfer_length: u32,
    time_out_value: u32,
    data_buffer_offset: u32,
    previous_task_file: [u8; 8],
    current_task_file: [u8; 8],
}

fn open_device(device_path: &str) -> Result<HANDLE> {
    let path_c = std::ffi::CString::new(device_path)
        .map_err(|_| SecDelError::InvalidParameter("Invalid device path".into()))?;

    let handle = unsafe {
        CreateFileA(
            windows::core::PCSTR(path_c.as_ptr() as *const u8),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(FILE_FLAG_NO_BUFFERING),
            None,
        )?
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(SecDelError::DeviceNotFound(format!(
            "Cannot open device: {}",
            device_path
        )));
    }

    Ok(handle)
}

fn close_handle(handle: HANDLE) {
    unsafe { CloseHandle(handle).ok() };
}

pub fn query_ata_info(device_path: &str) -> Result<Option<AtaIdentifyData>> {
    let handle = match open_device(device_path) {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };
    let result = read_identify_data_inner(handle);
    close_handle(handle);
    result
}

fn read_identify_data_inner(handle: HANDLE) -> Result<Option<AtaIdentifyData>> {
    let mut aptex = AtaPassThroughEx {
        length: mem::size_of::<AtaPassThroughEx>() as u16,
        ata_flags: 0,
        data_transfer_length: 512,
        time_out_value: 30,
        data_buffer_offset: mem::size_of::<AtaPassThroughEx>() as u32,
        previous_task_file: [0; 8],
        current_task_file: [0; 8],
    };
    aptex.current_task_file[0] = 0xEC;

    let mut output_buf = vec![0u8; mem::size_of::<AtaPassThroughEx>() + 512];
    unsafe {
        ptr::copy_nonoverlapping(
            &aptex as *const _ as *const u8,
            output_buf.as_mut_ptr(),
            mem::size_of::<AtaPassThroughEx>(),
        );
    }

    let mut bytes_returned: u32 = 0;
    let success = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_ATA_PASS_THROUGH,
            Some(output_buf.as_ptr() as *const std::ffi::c_void),
            output_buf.len() as u32,
            Some(output_buf.as_mut_ptr() as *mut std::ffi::c_void),
            output_buf.len() as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };

    if success.is_err() || bytes_returned < mem::size_of::<AtaPassThroughEx>() as u32 + 512 {
        return Err(SecDelError::AtaFailed("ATA IDENTIFY command failed".into()));
    }

    let data_start = mem::size_of::<AtaPassThroughEx>();
    let identify_buf = &output_buf[data_start..data_start + 512];

    let w0 = u16::from_le_bytes([identify_buf[0], identify_buf[1]]);
    if (w0 & 0x8000) != 0 {
        return Ok(None);
    }

    let w83 = u16::from_le_bytes([identify_buf[166], identify_buf[167]]);
    let w86 = u16::from_le_bytes([identify_buf[172], identify_buf[173]]);
    let w87 = u16::from_le_bytes([identify_buf[174], identify_buf[175]]);
    let w128 = u16::from_le_bytes([identify_buf[256], identify_buf[257]]);

    let secure_erase_supported = (w83 & 0x0010) != 0 && (w86 & 0x0010) != 0;
    let enhanced_erase_supported = (w83 & 0x0020) != 0 && (w87 & 0x0020) != 0;
    let is_frozen = (w128 & 0x0040) != 0;
    let w217 = u16::from_le_bytes([identify_buf[434], identify_buf[435]]);
    let is_ssd = w217 == 0x0001;

    fn swap_pairs(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(src.len());
        for pair in src.chunks(2) {
            if pair.len() == 2 {
                out.push(pair[1]);
                out.push(pair[0]);
            }
        }
        out
    }

    let model = String::from_utf8_lossy(&swap_pairs(&identify_buf[27..47]))
        .trim()
        .trim_matches('\0')
        .to_string();
    let firmware = String::from_utf8_lossy(&swap_pairs(&identify_buf[23..27]))
        .trim()
        .trim_matches('\0')
        .to_string();
    let serial = String::from_utf8_lossy(&swap_pairs(&identify_buf[10..20]))
        .trim()
        .trim_matches('\0')
        .to_string();

    Ok(Some(AtaIdentifyData {
        secure_erase_supported,
        enhanced_erase_supported,
        is_frozen,
        is_ssd,
        model,
        firmware,
        serial,
    }))
}

pub fn read_identify_data(device_path: &str) -> Result<AtaIdentifyData> {
    let handle = open_device(device_path)?;
    let result = query_ata_info(device_path)?;
    close_handle(handle);
    Ok(result.unwrap_or(AtaIdentifyData {
        secure_erase_supported: false,
        enhanced_erase_supported: false,
        is_frozen: false,
        is_ssd: false,
        model: String::new(),
        firmware: String::new(),
        serial: String::new(),
    }))
}

pub fn security_set_password(device_path: &str, password: &[u8]) -> Result<()> {
    let handle = open_device(device_path)?;

    let mut aptex = AtaPassThroughEx {
        length: mem::size_of::<AtaPassThroughEx>() as u16,
        ata_flags: 0,
        data_transfer_length: 512,
        time_out_value: 30,
        data_buffer_offset: mem::size_of::<AtaPassThroughEx>() as u32,
        previous_task_file: [0; 8],
        current_task_file: [0; 8],
    };
    aptex.current_task_file = [0xF1, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00];

    let mut data_buf = [0u8; 512];
    let pw_len = password.len().min(32);
    data_buf[2..2 + pw_len].copy_from_slice(&password[..pw_len]);

    let mut output_buf = vec![0u8; mem::size_of::<AtaPassThroughEx>() + 512];
    unsafe {
        ptr::copy_nonoverlapping(
            &aptex as *const _ as *const u8,
            output_buf.as_mut_ptr(),
            mem::size_of::<AtaPassThroughEx>(),
        );
        ptr::copy_nonoverlapping(
            data_buf.as_ptr(),
            output_buf
                .as_mut_ptr()
                .add(mem::size_of::<AtaPassThroughEx>()),
            512,
        );
    }

    let mut bytes_returned: u32 = 0;
    let result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_ATA_PASS_THROUGH,
            Some(output_buf.as_ptr() as *const std::ffi::c_void),
            output_buf.len() as u32,
            Some(output_buf.as_mut_ptr() as *mut std::ffi::c_void),
            output_buf.len() as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };

    close_handle(handle);
    if result.is_err() {
        return Err(SecDelError::AtaFailed(
            "ATA SECURITY SET PASSWORD command failed".into(),
        ));
    }
    Ok(())
}

pub fn security_erase_unit(device_path: &str, enhanced: bool) -> Result<()> {
    let handle = open_device(device_path)?;

    let mut aptex = AtaPassThroughEx {
        length: mem::size_of::<AtaPassThroughEx>() as u16,
        ata_flags: 0,
        data_transfer_length: 0,
        time_out_value: 300,
        data_buffer_offset: mem::size_of::<AtaPassThroughEx>() as u32,
        previous_task_file: [0; 8],
        current_task_file: [0; 8],
    };
    aptex.current_task_file = [
        0xF2,
        if enhanced { 0x11 } else { 0x01 },
        0x00,
        0x00,
        0x00,
        0x40,
        0x00,
        0x00,
    ];

    let mut output_buf = vec![0u8; mem::size_of::<AtaPassThroughEx>()];
    unsafe {
        ptr::copy_nonoverlapping(
            &aptex as *const _ as *const u8,
            output_buf.as_mut_ptr(),
            mem::size_of::<AtaPassThroughEx>(),
        );
    }

    let mut bytes_returned: u32 = 0;
    let result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_ATA_PASS_THROUGH,
            Some(output_buf.as_ptr() as *const std::ffi::c_void),
            output_buf.len() as u32,
            Some(output_buf.as_mut_ptr() as *mut std::ffi::c_void),
            output_buf.len() as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };

    close_handle(handle);
    if result.is_err() {
        return Err(SecDelError::AtaFailed(
            "ATA SECURITY ERASE UNIT command failed".into(),
        ));
    }
    Ok(())
}

pub fn check_tcg_opal(device_path: &str) -> Result<bool> {
    let _ = device_path;
    Ok(false)
}

pub fn opal_revert(device_path: &str, _psid: bool) -> Result<()> {
    let _ = device_path;
    Err(SecDelError::CryptoEraseFailed(
        "TCG Opal revert not implemented".into(),
    ))
}
