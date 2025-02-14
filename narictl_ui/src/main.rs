/*
Copyright d22592 and contributors
SPDX-License-Identifier: GPL-3.0-or-later
*/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows

use std::env;
use gtk::glib::clone;
use gtk::prelude::{BoxExt, ColorChooserExt, GtkWindowExt, RangeExt, WidgetExt};
use narictl_lib::*;
use relm4::{
    gtk, gtk::gdk::RGBA, gtk::Adjustment, ComponentParts, ComponentSender, RelmApp, SimpleComponent,
};

const CSS: &str = include_str!("application.css");
const BOX_SPACING: u32 = 22;

struct Application {
    device: Device,
    haptics: u8,
    mic_monitor: u8,
    color: (u8, u8, u8),
}

struct Widgets {
    ht_switch: gtk::Switch,
    hi_scale: gtk::Scale,
    mm_switch: gtk::Switch,
    mm_scale: gtk::Scale,
    c_effects_dropdown: gtk::DropDown,
    c_box: gtk::Box,
}

#[derive(Debug)]
enum AppInput {
    ToggleMicMonitor(bool),
    SetMicMonitor(u8),
    SetBrightness(u8),
    ToggleHaptics(bool),
    SetHapticIntensity(u8),
    SetColor(u8, u8, u8),
    SetState(u8),
}

impl SimpleComponent for Application {
    type Input = AppInput;
    type Output = ();
    type Init = u8;
    type Root = gtk::Window;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title(format!("NariCTL v{}", env!("CARGO_PKG_VERSION")))
            .default_width(500)
            .default_height(300)
            .build()
    }

    fn init(
        _counter: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let device = Device::init();
        if let Err(e) = device {
            eprintln!("Failed to initialize the headset. Error: {e}");
            std::process::exit(1);
        }

        let main = Application {
            device: device.unwrap(),
            haptics: 60,
            mic_monitor: 20,
            color: (0, 255, 0),
        };
        main.device.set_off();

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .vexpand(true)
            .hexpand(true)
            .css_name("main")
            .build();

        let main_stack = gtk::Stack::builder()
            .transition_duration(200)
            .hhomogeneous(false)
            .transition_type(gtk::StackTransitionType::SlideLeftRight)
            .css_name("main_stack")
            .build();

        let mv = main
            .device
            .get_voltage()
            .expect("Failed to get data from the headset");
        let info_text = if mv != 0 {
            format!(
                "<b>NariCTL</b> Version: {}\nVoltage: <i>{mv} mv</i>\n\n\n<small>Built and optimized for use with the Razer Nari Ultimate only</small>",
                env!("CARGO_PKG_VERSION")
            )
        } else {
            eprintln!("WARN: Unable to get voltage data");
            format!(
                "<b>NariCTL</b> Version: {}\n\n\n<small>Built and optimized for use with the Razer Nari Ultimate only</small>",
                env!("CARGO_PKG_VERSION")
            )
        };

        let about_label = gtk::Label::builder()
            .use_markup(true)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .label(info_text)
            .build();

        let device_label = gtk::Label::builder()
            .css_name("device_label")
            .label("Razer Nari Ultimate")
            .single_line_mode(true)
            .hexpand(true)
            .build();

        // HEADER -----------------------------
        let switcher_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(5)
            .hexpand(true)
            .homogeneous(true)
            .css_name("switcher_box")
            .build();

        let switcher = gtk::StackSwitcher::builder()
            .stack(&main_stack)
            .css_name("switcher")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        let header = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(5)
            .css_name("header")
            .build();

        let header_label = gtk::Label::builder()
            .label("General")
            .hexpand(false)
            .build();

        header.append(&header_label);
        switcher_box.append(&switcher);

        // ------------------------------------
        let f_box = gtk::FlowBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .column_spacing(BOX_SPACING)
            .row_spacing(BOX_SPACING)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .max_children_per_line(2)
            .min_children_per_line(1)
            .hexpand(true)
            .vexpand(true)
            .homogeneous(true)
            .css_name("property_box")
            .build();

        let f1_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .width_request(460)
            .css_name("properties")
            .build();

        let f2_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .width_request(512)
            .css_name("properties")
            .build();

        // Mic Monitoring Init
        let mm_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let mm_label = gtk::Label::builder()
            .css_name("title")
            .label("Mic Monitoring")
            .build();
        let mm_switch = gtk::Switch::builder()
            .valign(gtk::Align::Start)
            .halign(gtk::Align::Start)
            .build();

        mm_box.append(&mm_label);
        mm_box.append(&mm_switch);
        f1_box.append(&mm_box);

        let mm_scale = gtk::Scale::builder()
            .draw_value(true)
            .digits(0)
            .opacity(0.4)
            .sensitive(false)
            .show_fill_level(true)
            .adjustment(&Adjustment::new(
                main.mic_monitor as f64,
                0.,
                86.,
                1.,
                0.,
                0.,
            ))
            .build();
        let mm_scale_label = mm_scale.first_child().expect("Couldn't find the scale");
        mm_scale_label.set_margin_bottom(10);
        let mmi_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_name("info")
            .build();
        let mm_label_low = gtk::Label::builder()
            .label("0")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
        let mm_label_high = gtk::Label::builder()
            .label("86")
            .hexpand(true)
            .halign(gtk::Align::End)
            .build();

        mmi_box.append(&mm_label_low);
        mmi_box.append(&mm_label_high);
        f1_box.append(&mm_scale);
        f1_box.append(&mmi_box);

        f_box.append(&f1_box);
        f_box.append(&f2_box);

        // Haptics Toggle Init
        let ht_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let ht_label = gtk::Label::builder()
            .css_name("title")
            .label("Haptic Intensity")
            .build();
        let ht_switch = gtk::Switch::builder()
            .valign(gtk::Align::Start)
            .halign(gtk::Align::Start)
            .build();

        ht_box.append(&ht_label);
        ht_box.append(&ht_switch);
        f2_box.append(&ht_box);

        let hi_scale = gtk::Scale::builder()
            .digits(0)
            .draw_value(true)
            .adjustment(&Adjustment::new(main.haptics as f64, 20., 100., 1., 0., 0.))
            .show_fill_level(true)
            .opacity(0.4)
            .sensitive(false)
            .build();
        let hi_scale_label = hi_scale.first_child().expect("Couldn't find the scale");
        hi_scale_label.set_margin_bottom(10);
        let hi_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_name("info")
            .build();
        let hi_label_low = gtk::Label::builder()
            .label("20")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
        let hi_label_high = gtk::Label::builder()
            .label("100")
            .hexpand(true)
            .halign(gtk::Align::End)
            .build();

        hi_box.append(&hi_label_low);
        hi_box.append(&hi_label_high);
        f2_box.append(&hi_scale);
        f2_box.append(&hi_box);

        // ------------------------------------
        let lighting_box = gtk::FlowBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .column_spacing(BOX_SPACING)
            .row_spacing(BOX_SPACING)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .max_children_per_line(2)
            .min_children_per_line(1)
            .hexpand(true)
            .vexpand(true)
            .css_name("property_box")
            .build();

        let l1_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .width_request(512)
            .css_name("properties")
            .build();

        let l2_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::Start)
            .width_request(512)
            .css_name("properties")
            .build();

        // Brightness Init
        let b_label = gtk::Label::builder()
            .css_name("title")
            .halign(gtk::Align::Start)
            .label("Brightness")
            .build();
        let b_scale = gtk::Scale::builder()
            .digits(0)
            .draw_value(true)
            .show_fill_level(true)
            .adjustment(&Adjustment::new(60., 0., 100., 2., 0., 0.))
            .build();
        let b_scale_label = b_scale.first_child().expect("Couldn't find the scale");
        b_scale_label.set_margin_bottom(10);
        let b_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_name("info")
            .spacing(6)
            .build();
        let b_label_low = gtk::Label::builder()
            .label("0")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .margin_top(10)
            .build();
        let b_label_high = gtk::Label::builder()
            .label("100")
            .hexpand(true)
            .halign(gtk::Align::End)
            .margin_top(10)
            .build();

        b_box.append(&b_label_low);
        b_box.append(&b_label_high);

        l1_box.append(&b_label);
        l1_box.append(&b_scale);
        l1_box.append(&b_box);

        let c_label = gtk::Label::builder()
            .label("Effects")
            .halign(gtk::Align::Start)
            .css_name("title")
            .build();

        let c_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();

        let c_effects_dropdown = gtk::DropDown::from_strings(&["Off", "Static"]);
        c_effects_dropdown.set_halign(gtk::Align::Start);
        c_effects_dropdown.set_width_request(170);

        let c_colorpicker_label = gtk::Label::builder()
            .label("Color")
            .halign(gtk::Align::Start)
            .css_name("c0")
            .build();
        let c_colorpicker = gtk::ColorButton::builder()
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Start)
            .title("Set Color")
            .build();
        c_colorpicker.set_size_request(1, 1);
        let swatch = c_colorpicker
            .first_child()
            .expect("Couldn't find the colorbutton")
            .first_child()
            .expect("Couldn't find the color swatch");
        swatch.set_width_request(20);
        c_colorpicker.set_rgba(&RGBA::new(0., 1., 0., 1.));
        c_box.append(&c_colorpicker_label);
        c_box.append(&c_colorpicker);
        c_box.set_sensitive(false);
        c_box.set_opacity(0.);

        l2_box.append(&c_label);
        l2_box.append(&c_effects_dropdown);
        l2_box.append(&c_box);
        lighting_box.append(&l1_box);
        lighting_box.append(&l2_box);

        main_stack.add_titled(&f_box, Some("general"), "General");
        main_stack.add_titled(&lighting_box, Some("lighting"), "Lighting");
        main_stack.add_titled(&about_label, Some("about"), "About");

        main_box.append(&switcher_box);
        main_box.append(&main_stack);
        main_box.append(&device_label);

        window.set_child(Some(&main_box));

        // Listeners
        b_scale.connect_value_changed(clone!(@strong sender => move |v| {
            sender.input(AppInput::SetBrightness(v.value() as u8));
        }));

        mm_scale.connect_value_changed(clone!(@strong sender => move |v| {
            sender.input(AppInput::SetMicMonitor(v.value() as u8));
        }));

        hi_scale.connect_value_changed(clone!(@strong sender => move |v| {
            sender.input(AppInput::SetHapticIntensity(v.value() as u8));
        }));

        ht_switch.connect_state_notify(clone!(@strong sender => move |t| {
            sender.input(AppInput::ToggleHaptics(t.state()));
        }));

        mm_switch.connect_state_notify(clone!(@strong sender => move |t| {
            sender.input(AppInput::ToggleMicMonitor(t.state()));
        }));

        c_colorpicker.connect_rgba_notify(clone!(@strong sender => move |c| {
            let c = c.rgba();
            let (r, g, b) = ((c.red() * 255.99) as u8, (c.green() * 255.99) as u8, (c.blue() * 255.99) as u8);
            sender.input(AppInput::SetColor(r, g, b));
        }));

        c_effects_dropdown.connect_selected_item_notify(clone!(@strong sender => move |d| {
            sender.input(AppInput::SetState(d.selected() as u8));
        }));

        ComponentParts {
            model: main,
            widgets: Widgets {
                ht_switch,
                hi_scale,
                mm_switch,
                mm_scale,
                c_effects_dropdown,
                c_box,
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::ToggleMicMonitor(state) => {
                let value = if state { self.mic_monitor } else { 0 };
                if let Err(e) = self.device.set_mic_monitor(value) {
                    eprintln!("An error occurred while trying to change the mic monitor state, Error: {e}");
                }
            }

            AppInput::SetMicMonitor(mut value) => {
                value /= 2;
                if value % 2 != 0 {
                    value -= 1;
                }
                if let Err(e) = self.device.set_mic_monitor(value + 192) {
                    eprintln!("An error occurred while trying to change the mic monitor state, Error: {e}");
                } else {
                    self.mic_monitor = value + 192;
                }
            }

            AppInput::SetBrightness(brightness) => {
                if let Err(e) = self.device.set_brightness(brightness) {
                    eprintln!(
                        "An error occurred while trying to change the brightness, Error: {e}"
                    );
                }
            }

            AppInput::SetHapticIntensity(mut intensity) => {
                if intensity > 100 {
                    intensity = 100;
                }
                if let Err(e) = self.device.set_haptic_intensity(intensity, true) {
                    eprintln!("An error occurred while trying to set haptic intensity, Error: {e}");
                } else {
                    self.haptics = intensity;
                }
            }

            AppInput::ToggleHaptics(state) => {
                if let Err(e) = self.device.set_haptic_intensity(self.haptics, state) {
                    eprintln!("An error occurred while trying toggle haptics, Error: {e}");
                }
            }

            AppInput::SetColor(r, g, b) => {
                if let Err(e) = self.device.set_fixed(r, g, b) {
                    eprintln!("An error occurred while trying to change the color, Error: {e}");
                } else {
                    self.color = (r, g, b);
                }
            }

            AppInput::SetState(state) => {
                let res = match state {
                    0 => self.device.set_off(),
                    1 => self
                        .device
                        .set_fixed(self.color.0, self.color.1, self.color.2),
                    _ => {
                        eprintln!("Invalid state, ignoring");
                        Ok(0)
                    }
                };
                if let Err(e) = res {
                    eprintln!("An error occurred while trying to change the state, Error: {e}");
                }
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        if widgets.ht_switch.state() {
            widgets.hi_scale.set_sensitive(true);
            widgets.hi_scale.set_opacity(1.);
        } else {
            widgets.hi_scale.set_sensitive(false);
            widgets.hi_scale.set_opacity(0.4);
        }
        if widgets.mm_switch.state() {
            widgets.mm_scale.set_sensitive(true);
            widgets.mm_scale.set_opacity(1.);
        } else {
            widgets.mm_scale.set_sensitive(false);
            widgets.mm_scale.set_opacity(0.4);
        }
        if widgets.c_effects_dropdown.selected() == 0 {
            widgets.c_box.set_sensitive(false);
            widgets.c_box.set_opacity(0.);
        } else {
            widgets.c_box.set_sensitive(true);
            widgets.c_box.set_opacity(1.);
        }
    }
}

fn main() {
    let app = RelmApp::new("me.d22592.narictl");
    relm4::set_global_css(CSS);
    app.run::<Application>(0);
}
