use crate::error::{Result, SecDelError};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;
const FSCTL_FILE_LEVEL_TRIM: u32 = 0x00098248;
const FSCTL_SET_SPARSE: u32 = 0x000900C4;
const FSCTL_SET_ZERO_DATA: u32 = 0x000980C8;

#[repr(C)]
struct FileLevelTrimRange {
    offset: i64,
    length: i64,
}

#[repr(C)]
struct FileLevelTrimOutput {
    num_ranges_processed: u32,
    num_ranges_failed: u32,
    num_ranges: u32,
    total_bytes_trimmed: u64,
    unused: u32,
}

#[repr(C)]
struct FileZeroDataInformation {
    file_offset: i64,
    beyond_final_zero: i64,
}

fn open_file_wide(path: &Path, flags: u32) -> Result<HANDLE> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(flags),
            None,
        )?
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(SecDelError::TrimFailed(format!(
            "Cannot open file: {}",
            path.display()
        )));
    }
    Ok(handle)
}

fn open_device_wide(device_path: &str, flags: u32) -> Result<HANDLE> {
    let wide: Vec<u16> = device_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(flags),
            None,
        )?
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(SecDelError::TrimFailed(format!(
            "Cannot open device: {}",
            device_path
        )));
    }
    Ok(handle)
}

fn close_handle(handle: HANDLE) {
    unsafe { CloseHandle(handle).ok() };
}

pub fn trim_file(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let file_len = metadata.len() as i64;
    let handle = open_file_wide(path, FILE_FLAG_NO_BUFFERING)?;

    let range = FileLevelTrimRange {
        offset: 0,
        length: file_len,
    };
    let mut output = FileLevelTrimOutput {
        num_ranges_processed: 0,
        num_ranges_failed: 0,
        num_ranges: 0,
        total_bytes_trimmed: 0,
        unused: 0,
    };
    let mut bytes_returned: u32 = 0;
    let result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            FSCTL_FILE_LEVEL_TRIM,
            Some(&range as *const _ as *const std::ffi::c_void),
            mem::size_of::<FileLevelTrimRange>() as u32,
            Some(&mut output as *mut _ as *mut std::ffi::c_void),
            mem::size_of::<FileLevelTrimOutput>() as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };
    close_handle(handle);
    match result {
        Ok(()) => Ok(output.total_bytes_trimmed / 512),
        Err(e) => Err(SecDelError::TrimFailed(format!(
            "FSCTL_FILE_LEVEL_TRIM failed: {}",
            e
        ))),
    }
}

pub fn mark_sparse_and_zero(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let file_len = metadata.len() as i64;
    let handle = open_file_wide(path, FILE_ATTRIBUTE_NORMAL)?;

    let mut bytes_returned: u32 = 0;
    let sparse_result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            FSCTL_SET_SPARSE,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };
    if sparse_result.is_err() {
        close_handle(handle);
        return Err(SecDelError::TrimFailed("FSCTL_SET_SPARSE failed".into()));
    }

    let zero_data = FileZeroDataInformation {
        file_offset: 0,
        beyond_final_zero: file_len,
    };
    bytes_returned = 0;
    let zero_result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            FSCTL_SET_ZERO_DATA,
            Some(&zero_data as *const _ as *const std::ffi::c_void),
            mem::size_of::<FileZeroDataInformation>() as u32,
            None,
            0,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };
    close_handle(handle);
    match zero_result {
        Ok(()) => Ok((file_len / 512) as u64),
        Err(e) => Err(SecDelError::TrimFailed(format!(
            "FSCTL_SET_ZERO_DATA failed: {}",
            e
        ))),
    }
}

pub fn trim_device(device_path: &str) -> Result<u64> {
    let handle = open_device_wide(device_path, FILE_FLAG_NO_BUFFERING)?;
    let range = FileLevelTrimRange {
        offset: 0,
        length: -1i64,
    };
    let mut output = FileLevelTrimOutput {
        num_ranges_processed: 0,
        num_ranges_failed: 0,
        num_ranges: 0,
        total_bytes_trimmed: 0,
        unused: 0,
    };
    let mut bytes_returned: u32 = 0;
    let result = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            FSCTL_FILE_LEVEL_TRIM,
            Some(&range as *const _ as *const std::ffi::c_void),
            mem::size_of::<FileLevelTrimRange>() as u32,
            Some(&mut output as *mut _ as *mut std::ffi::c_void),
            mem::size_of::<FileLevelTrimOutput>() as u32,
            Some(&mut bytes_returned as *mut u32),
            None,
        )
    };
    close_handle(handle);
    match result {
        Ok(()) => Ok(output.total_bytes_trimmed / 512),
        Err(e) => Err(SecDelError::TrimFailed(format!(
            "Device TRIM failed: {}",
            e
        ))),
    }
}
