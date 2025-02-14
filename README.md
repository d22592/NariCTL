# NariCTL

NariCTL is a tool which allows you to make changes to the Razer Nari Ultimate headset settings

## Installation

### Releases

Download the executable for your platform from the Releases page and execute the program

### Building

Get rust for your platform by going to [rustup](https://rustup.rs) and following the instructions in the website.

Once rust is installed, clone the repository, change the directory to the root of the cloned repository and run
`cargo b --release`

Once the build completes, try out the completed binaries at "target/release" directory

Note: If you get permission errors, copy `71-narictl.rules` to `/usr/lib/udev/rules.d/`, then run this command as the root user: `udevadm control --reload-rules && udevadm trigger`

## Features

- Enable/Disable haptics
- Set haptics intensity level
- Enable mic monitoring
- Set mic monitoring volume level
- Set LED color in the headset (static only)
- Set LED brightness
- Get headset voltage (Experimental)

## List of features that will be added in future releases

- Add support for reading data from the headset
- Add more color effects support (fading, spectrum, etc)

## This project was possible from these great libraries

- rust (https://www.rust-lang.org)

- relm 4 (https://relm4.org)

- gtk-rs (https://gtk-rs.org)

- rusb (https://github.com/a1ien/rusb)

## Disclaimer

RAZER is the trademark or registered trademark of Razer Inc.