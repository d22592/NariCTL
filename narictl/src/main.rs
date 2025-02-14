/*
Copyright d22592 and contributors
SPDX-License-Identifier: GPL-3.0-or-later
*/

use narictl_lib::*;
use std::env;

const GENERIC_ERROR: &str = "Could not communicate with the headset";

fn main() {
    let device = Device::init().expect("Unable to initialize the headset");
    let arguments: Vec<String> = env::args().collect();

    match arguments.get(1).unwrap_or(&String::new()).as_str() {
        "color" => {
            let args = arguments.get(2);
            let rgb = args
                .unwrap_or(&String::new())
                .split(',')
                .map(|color| color.parse::<u8>().unwrap_or_default())
                .collect::<Vec<u8>>();
            if rgb.len() != 3 {
                eprintln!("Invalid value, (ex: 255,0,0)");
                return;
            }
            device
                .set_fixed(rgb[0], rgb[1], rgb[2])
                .expect(GENERIC_ERROR);
        }
        "brightness" => {
            let percentage = arguments
                .get(2)
                .unwrap_or(&String::new())
                .parse::<u8>()
                .unwrap_or_default();
            if percentage > 100 {
                eprintln!("[WARN]: percentage should not be more than 100.");
            }
            device
                .set_brightness(percentage)
                .expect(GENERIC_ERROR);
        }
        "haptics" => {
            let percentage = arguments
                .get(2)
                .unwrap_or(&String::new())
                .parse::<u8>()
                .unwrap_or_default();
            if percentage > 100 {
                eprintln!("[WARN]: percentage should be more than 100.");
            }
            device
                .set_haptic_intensity(percentage, true)
                .expect(GENERIC_ERROR);
        }
        "mic_monitor" | "sidetone" => {
            let mut enable = arguments
                .get(2)
                .unwrap_or(&String::default())
                .parse::<u8>()
                .unwrap_or_default();
            if enable > 88 {
                eprintln!("[WARN]: value should not be more than 88.");
                enable = 0;
            }
            enable /= 2;
            if enable % 2 != 0 {
                enable -= 1;
            }
            enable += 192;
            device
                .set_mic_monitor(enable)
                .expect(GENERIC_ERROR);
        }
        "voltage" | "mv" => println!("Voltage: {} mv", device.get_voltage().expect(GENERIC_ERROR)),
        "--help" | "-h" => help(),
        "--version" | "-v" => println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        cmd => {
            eprintln!("Invalid choice: {cmd}");
            help();
        }
    }
}

fn help() {
    println!(r#"Usage: {} [options] [arguments]

Options:
    color <R,G,B>                     | Change led color to values (max value for each 255)
    brightness <value>                | Change led brighness to value (max 100)
    haptics <value>                   | Change haptic intensity to value (max 100)
    mic_monitor, sidetone <value>     | Change mic monitor state to value (max 86)
    mv, voltage                       | Get the current voltage of the battery (EXPERIMENTAL)
    -h, --help                        | Shows this help menu
    -v, --version                     | Shows the application's version 
"#,
        env!("CARGO_PKG_NAME")
    );
}
