use crate::error::Result;
use std::collections::HashMap;
use windows::core::BSTR;
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WBEM_FLAG_FORWARD_ONLY,
    WBEM_FLAG_RETURN_IMMEDIATELY,
};

pub fn query_wmi(query: &str) -> Result<Vec<HashMap<String, Option<String>>>> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let result = wmi_query_inner(query);
    unsafe {
        CoUninitialize();
    }
    result
}

fn wmi_query_inner(query: &str) -> Result<Vec<HashMap<String, Option<String>>>> {
    unsafe {
        let locator: IWbemLocator = windows::Win32::System::Com::CoCreateInstance(
            &windows::Win32::System::Wmi::WbemLocator,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        let services: IWbemServices = locator.ConnectServer(
            &BSTR::from("root\\cimv2"),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            0,
            &BSTR::new(),
            None,
        )?;

        let enumerator = services.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from(query),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )?;

        let mut results = Vec::new();

        loop {
            let mut objects = vec![Option::<IWbemClassObject>::None];
            let mut returned: u32 = 0;
            let hr = enumerator.Next(
                windows::Win32::System::Wmi::WBEM_INFINITE,
                &mut objects,
                &mut returned,
            );
            if hr.is_err() || returned == 0 {
                break;
            }

            if let Some(ref obj) = objects[0] {
                let mut item = HashMap::new();
                let known_props = [
                    "Model",
                    "DeviceID",
                    "Size",
                    "InterfaceType",
                    "MediaType",
                    "PNPDeviceID",
                    "DeviceObject",
                    "InstallDate",
                    "DriveLetter",
                    "Caption",
                    "Name",
                    "SerialNumber",
                    "FirmwareRevision",
                    "Index",
                    "Partitions",
                    "Status",
                ];
                for &prop in &known_props {
                    let bstr_prop = BSTR::from(prop);
                    let mut variant = VARIANT::default();
                    if obj.Get(&bstr_prop, 0, &mut variant, None, None).is_ok() {
                        let val = variant_to_string(&variant);
                        item.insert(prop.to_string(), val);
                    }
                }
                results.push(item);
            }
        }
        Ok(results)
    }
}

fn variant_to_string(v: &VARIANT) -> Option<String> {
    unsafe {
        let vt = v.Anonymous.Anonymous.vt;
        if vt == windows::Win32::System::Variant::VARENUM(0)
            || vt == windows::Win32::System::Variant::VARENUM(0x0001)
        {
            return None;
        }
        let bstr_val = &v.Anonymous.Anonymous.Anonymous.bstrVal;
        let ptr = *(bstr_val as *const _ as *const *const u16);
        if ptr.is_null() {
            return None;
        }
        let s = BSTR::from_raw(ptr);
        Some(s.to_string())
    }
}
