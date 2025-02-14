/*
Copyright d22592 and contributors
SPDX-License-Identifier: GPL-3.0-or-later
*/

use rusb::{Context, DeviceHandle, Error, UsbContext};
use std::time::Duration;

const HID: u8 = 5;
const VID: u16 = 0x1532;
const PID: u16 = 0x051a;
const TIMEOUT: Duration = Duration::from_secs(4);

pub struct Device {
    hid_handle: DeviceHandle<Context>,
}

impl Device {
    pub fn init() -> Result<Self, String> {
        match Context::new() {
            Ok(mut context) => match open_device(&mut context) {
                Some(handle) => {
                    if handle.kernel_driver_active(HID).is_ok_and(|yes| yes) {
                        if let Err(e) = handle.detach_kernel_driver(HID) {
                            if e != Error::NotSupported && e != Error::NotFound {
                                return Err(String::from("Failed to detach kernel driver"));
                            }
                        }
                    }
                    if handle.claim_interface(HID).is_ok() {
                        Ok(Device { hid_handle: handle })
                    } else {
                        Err(String::from("Failed to claim interface"))
                    }
                }
                None => Err(format!("Failed to find device {VID:04x}:{PID:04x}")),
            },
            Err(_) => Err(String::from("Failed to open device")),
        }
    }

    // GENERAL
    pub fn set_mic_monitor(&self, mut value: u8) -> Result<usize, Error> {
        if !(0xc0..0xec).contains(&value) {
            value = 0xc0;
        }
        let result = decode(format!("ff0a00ff040ef105010400{value:02x}")).unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)
    }

    pub fn set_haptic_intensity(&self, mut percentage: u8, enable: bool) -> Result<usize, Error> {
        if percentage > 100 {
            percentage = 0;
        }
        let result = decode(format!(
            "ff0a00ff0402f10620{:02x}{:02x}",
            enable as u8, percentage
        ))
        .unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)
    }

    // LIGHTING
    pub fn set_brightness(&self, mut percentage: u8) -> Result<usize, Error> {
        if percentage > 100 {
            percentage = 0;
        }
        let result = decode(format!("ff0a00ff0412f10371{percentage:02x}")).unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)
    }

    pub fn set_off(&self) -> Result<usize, Error> {
        let result = decode("ff0a00ff0412f10572").unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)
    }

    pub fn set_fixed(&self, r: u8, g: u8, b: u8) -> Result<usize, Error> {
        let result = decode(format!("ff0a00ff0412f10572{r:02x}{g:02x}{b:02x}")).unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)
    }

    // EXTRAS (Reading data from the headset is experimental)
    pub fn get_voltage(&self) -> Result<u16, Error> {
        let result = decode("ff0a00fd0412f10205").unwrap();
        self.hid_handle
            .write_control(0x21, 9, 0x03ff, 5, &result, TIMEOUT)?;

        let mut voltage_res = [0; 64];
        self.hid_handle
            .read_control(0xa1, 1, 0x03ff, 5, &mut voltage_res, TIMEOUT)?;

        let result = u16::from_be_bytes([voltage_res[12], voltage_res[13]]);

        if result == 0 {
            let trying_again = self.get_voltage()?;
            if trying_again == 0 {
                return Err(Error::Other)
            }
            return Ok(trying_again)
        }
        Ok(result)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if self.hid_handle.release_interface(HID).is_ok() {
            if let Err(e) = self.hid_handle.attach_kernel_driver(HID) {
                if e != Error::NotFound {
                    eprintln!("[WARN]: Failed to attach kernel driver");
                }
            }
        } else {
            eprintln!("[WARN]: Failed to release interface");
        }
    }
}

fn decode<T: Into<String>>(data: T) -> Option<Vec<u8>> {
    let mut vector = Vec::new();
    let data = data.into();

    for bytes in data.as_bytes().chunks(2) {
        let chunk = std::str::from_utf8(bytes).expect("Failed to decode string, must be UTF-8");
        let new_byte = u8::from_str_radix(chunk, 16).expect("Unable to convert chunk to int");
        vector.push(new_byte);
    }
    if vector.is_empty() || vector.len() < data.len() / 2 || data.len() % 2 != 0 {
        None
    } else {
        while vector.len() != 64 {
            vector.push(0);
        }
        Some(vector)
    }
}

fn open_device(context: &mut Context) -> Option<DeviceHandle<Context>> {
    match context.devices() {
        Ok(device_list) => for device in device_list.iter() {
            if let Ok(device_desc) = device.device_descriptor() {
                if device_desc.vendor_id() == VID && device_desc.product_id() == PID {
                    println!("Found headset, attempting to open device");
                    match device.open() {
                        Ok(handle) => return Some(handle),
                        Err(e) => eprintln!("Device found but failed to open: {e}"),
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to get device list: {e}"),
    }
    None
}
