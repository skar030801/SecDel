use crate::error::{Result, SecDelError};
use std::mem;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileA, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const IOCTL_STORAGE_PROTOCOL_COMMAND: u32 = 0x002D1400;
const PROTOCOL_TYPE_NVME: u32 = 0x04;
const STORAGE_PROTOCOL_COMMAND_FLAG_ADAPTER_REQUEST: u32 = 0x80000000;

#[derive(Debug, Clone)]
pub struct NvmeQueryInfo {
    pub version: String,
    pub sanitize_capable: bool,
    pub format_nvm_supported: bool,
    pub secure_erase_supported: bool,
    pub namespace_count: u32,
    pub temperature: Option<f64>,
    pub wear_level: Option<u32>,
}

#[repr(C, packed)]
struct StorageProtocolCommand {
    version: u32,
    length: u32,
    protocol_type: u32,
    flags: u32,
    command: u32,
    error_code: u32,
    return_code: u32,
    reserved: u32,
    duration: u64,
    data_buffer_length: u32,
    specific: [u32; 4],
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
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(SecDelError::DeviceNotFound(format!(
            "Cannot open NVMe device: {}",
            device_path
        )));
    }
    Ok(handle)
}

fn close_handle(handle: HANDLE) {
    unsafe { CloseHandle(handle).ok() };
}

pub fn query_nvme_info(device_path: &str) -> Result<Option<NvmeQueryInfo>> {
    let handle = match open_device(device_path) {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };
    let result = query_nvme_info_inner(handle);
    close_handle(handle);
    result
}

fn query_nvme_info_inner(handle: HANDLE) -> Result<Option<NvmeQueryInfo>> {
    let spc = StorageProtocolCommand {
        version: 1,
        length: mem::size_of::<StorageProtocolCommand>() as u32,
        protocol_type: PROTOCOL_TYPE_NVME,
        flags: STORAGE_PROTOCOL_COMMAND_FLAG_ADAPTER_REQUEST,
        command: 0x06,
        error_code: 0,
        return_code: 0,
        reserved: 0,
        duration: 0,
        data_buffer_length: 4096,
        specific: [0; 4],
    };

    let total_size = mem::size_of::<StorageProtocolCommand>() + 4096;
    let mut output_buf = vec![0u8; total_size];
    unsafe {
        ptr::copy_nonoverlapping(
            &spc as *const _ as *const u8,
            output_buf.as_mut_ptr(),
            mem::size_of::<StorageProtocolCommand>(),
        );
    }

    let mut bytes_returned: u32 = 0;
    let success = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_STORAGE_PROTOCOL_COMMAND,
            Some(output_buf.as_ptr() as *const std::ffi::c_void),
            total_size as u32,
            Some(output_buf.as_mut_ptr() as *mut std::ffi::c_void),
            total_size as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };
    if success.is_err() {
        return Ok(None);
    }

    let data_start = mem::size_of::<StorageProtocolCommand>();
    if bytes_returned < data_start as u32 + 4096 {
        return Ok(None);
    }

    let data = &output_buf[data_start..data_start + 4096];
    let vid = u16::from_le_bytes([data[0], data[1]]);
    if vid == 0 {
        return Ok(None);
    }

    let nn = u32::from_le_bytes([data[260], data[261], data[262], data[263]]);
    let ver_raw = u32::from_le_bytes([data[272], data[273], data[274], data[275]]);
    let version = format!(
        "{}.{}.{}",
        (ver_raw >> 16) as u16,
        (ver_raw >> 8) as u8,
        ver_raw as u8
    );

    let oacs = u16::from_le_bytes([data[512], data[513]]);
    let format_nvm_supported = (oacs & 0x0004) != 0;
    let sanitize_supported = (oacs & 0x0010) != 0;

    let temp_raw = u16::from_le_bytes([data[516], data[517]]);
    let temperature = if temp_raw > 0 {
        Some((temp_raw as f64) - 273.15)
    } else {
        None
    };

    Ok(Some(NvmeQueryInfo {
        version,
        sanitize_capable: sanitize_supported,
        format_nvm_supported,
        secure_erase_supported: format_nvm_supported,
        namespace_count: nn,
        temperature,
        wear_level: None,
    }))
}

fn send_nvme_admin_command(device_path: &str, command_code: u32, specific: [u32; 4]) -> Result<()> {
    let handle = open_device(device_path)?;
    let spc = StorageProtocolCommand {
        version: 1,
        length: mem::size_of::<StorageProtocolCommand>() as u32,
        protocol_type: PROTOCOL_TYPE_NVME,
        flags: STORAGE_PROTOCOL_COMMAND_FLAG_ADAPTER_REQUEST,
        command: command_code,
        error_code: 0,
        return_code: 0,
        reserved: 0,
        duration: 0,
        data_buffer_length: 0,
        specific,
    };

    let mut output_buf = vec![0u8; mem::size_of::<StorageProtocolCommand>()];
    unsafe {
        ptr::copy_nonoverlapping(
            &spc as *const _ as *const u8,
            output_buf.as_mut_ptr(),
            mem::size_of::<StorageProtocolCommand>(),
        );
    }

    let mut bytes_returned: u32 = 0;
    let result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_STORAGE_PROTOCOL_COMMAND,
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
        return Err(SecDelError::NvmeFailed("NVMe admin command failed".into()));
    }
    Ok(())
}

pub fn send_sanitize_command(device_path: &str, action: u8) -> Result<()> {
    send_nvme_admin_command(device_path, 0x81, [action as u32, 0, 0, 0])
}

pub fn send_format_nvm_command(device_path: &str, secure_erase: bool) -> Result<()> {
    let ses: u32 = if secure_erase { 1 } else { 0 };
    send_nvme_admin_command(device_path, 0x80, [ses << 8, 0, 0, 0])
}
