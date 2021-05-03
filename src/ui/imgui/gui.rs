use imgui::*;
use rtlsdr::USBStrings;
use std::collections::VecDeque;
use imgui::ImString;
use rtlsdr::RTLSDRError;
use crate::scanner::{Scanner, ScannerStatus};
use futures::sync::mpsc::UnboundedReceiver;
use std::sync::{Arc, Mutex};
use crate::{SAMPLERATE, DWELL_MS, BANDWIDTH};

const LOG_LEN: usize = 100;

pub(crate) struct State {
    pub show_log: bool,
    // TODO: do colors for errors
    pub log: VecDeque<String>,
    pub devices: Vec<Device>,
    pub selected_device: usize,
    pub scan_from: u32,
    pub scan_to: u32,
    pub is_running: bool,
    pub scanner_cmd: Option<Arc<Mutex<VecDeque<ScannerStatus>>>>,
    pub data: Vec<f32>,
}

pub(crate) struct Device {
    pub name: String,
    pub usb_description: USBStrings,
    pub gains: Vec<ImString>,
    pub selected_gain: i32,
    pub tuner_type: String,
}

impl State {
    pub fn new() -> Self {
        State {
            show_log: false,
            log: VecDeque::with_capacity(100),
            devices: vec![],
            selected_device: 0,
            scan_from: 60e6 as u32,
            scan_to: 1700e6 as u32,
            is_running: false,
            scanner_cmd: None,
            data: vec![],
        }
    }

    pub fn append_log(&mut self, str: String) {
        while self.log.len() > LOG_LEN - 1 {
            self.log.pop_front();
        }
        self.log.push_back(str);
    }
}

impl Device {
    pub fn probe(idx: i32) -> Result<Self, RTLSDRError> {
        let mut dev = rtlsdr::open(idx)?;
        let name = rtlsdr::get_device_name(idx);
        let usb_description = rtlsdr::get_device_usb_strings(idx)?;
        let gains = dev.get_tuner_gains()?.iter().
            // Gain is reported in 10th of Db
            map(|gain| ImString::new(format!("{}", (*gain as f32)/10.0))).collect::<Vec<_>>();
        let (_, tuner_type) = dev.get_tuner_type();
        Ok(Device{ name, usb_description, gains, selected_gain: 0, tuner_type })
    }
}

fn process_scanner_events(state: &mut Arc<Mutex<State>>, (width,height): (f32,f32)) {
    let mut state = state.lock().unwrap();
    let mut scanner_cmd = None;
    ::std::mem::swap(&mut scanner_cmd, &mut state.scanner_cmd);

    if let Some(scanner_cmd) = scanner_cmd.as_mut() {
        for cmd in scanner_cmd.lock().unwrap().drain(..) {
            match cmd {
                ScannerStatus::Info(msg) => {
                    info!("{}", msg);
                    state.append_log(format!("INFO {}", msg));
                },
                ScannerStatus::Error(msg) => {
                    error!("{}", msg);
                    state.append_log(format!("ERROR {}", msg));
                },
                ScannerStatus::Complete => {
                    state.is_running = false;
                    info!("Scanner complete")
                },
                ScannerStatus::Data(data) => {
                    let mut data = data.into_iter().map(|d| d as f32).collect();
                    state.data.append(&mut data);
                },
            }
        }
    }
    ::std::mem::swap(&mut state.scanner_cmd, &mut scanner_cmd);
}

pub(crate) fn render(ui: &Ui, state: &mut Arc<Mutex<State>>) -> bool {
    // TODO: should be integrated into support_gfx
    process_scanner_events(state, ui.get_window_size());

    let main_styles = vec![StyleVar::WindowRounding(0.0), StyleVar::WindowMinSize(ImVec2::new(200.0, 100.0))];
    ui.with_style_vars(&main_styles, ||{
        ui.window(im_str!("_Main"))
            .size(ui.imgui().display_size(), ImGuiCond::Always)
            .position((0.0, 0.0), ImGuiCond::FirstUseEver)
            .resizable(false)
            .collapsible(false)
            .movable(false)
            .title_bar(false)
            .build(|| {
                render_scan(&ui, &state);
                ui.separator();

                render_settings(&ui, &state);
                ui.separator();

                render_full_view(&ui, &state);
            });
    });


    true
}

fn render_scan(ui: &Ui, state: &Arc<Mutex<State>>) {
    if ui.collapsing_header(im_str!("Scan")).build() {
        let mut state = state.lock().unwrap();
        let mut from = state.scan_from as f32 / 1e6;
        let mut to = state.scan_to as f32 / 1e6;
        ui.input_float(im_str!("From"), &mut from).
            step(0.01).
            step_fast(1.0).
            build();
        ui.input_float(im_str!("To"), &mut to).
            step(0.01).
            step_fast(1.0).
            build();

        if state.is_running {
            if ui.small_button(im_str!("Stop")) {
                /*let mut rx_data = None;
                mem::swap(&mut state.rx_data, &mut rx_data);
                if let Some(mut rx_data) = rx_data {
                    debug!("UI closed receiver");
                };*/
                state.is_running = false;
            }
        } else {
            if ui.small_button(im_str!("Start")) {
                state.is_running = true;
                let scanner = Scanner::new(
                    state.selected_device as i32,
                    SAMPLERATE,
                    state.scan_from,
                    state.scan_to,
                    DWELL_MS,
                    BANDWIDTH
                );
                let rx_data = scanner.start();
                state.scanner_cmd = Some(rx_data);
            }
        }

        let from = (from * 1e6) as u32;
        let to = (to * 1e6) as u32;
        if state.scan_from != from {
            state.scan_from = from;
        }
        if state.scan_to != to {
            state.scan_to = to;
        }
    };
}

fn render_settings(ui: &Ui, state: &Arc<Mutex<State>>) {
    if ui.collapsing_header(im_str!("Settings")).build() {
        let state = &mut state.lock().unwrap();

        ui.tree_node(im_str!("Devices")).build(|| {
            let mut selected_device = state.selected_device;
            for (idx, device) in state.devices.iter_mut().enumerate() {
                //
                // Device
                //
                ui.tree_node(im_str!("{} {}", idx+1, device.name)).build(|| {

                    let mut selected = selected_device == idx;
                    if ui.checkbox(im_str!("Input"), &mut selected) {
                        selected_device = idx;
                    }

                    ui.text(im_str!("Manufacturer: {}", device.usb_description.manufacturer));
                    ui.text(im_str!("Product: {}", device.usb_description.product));
                    ui.text(im_str!("Serial: {}", device.usb_description.serial));
                    ui.text(im_str!("Device type: {}", device.tuner_type));

                    //
                    // Gain
                    //
                    ui.with_item_width(70.0, || {
                        let gains = device.gains.iter().map(|gain| gain.as_ref()).collect::<Vec<_>>();
                        ui.combo(im_str!("Gain (dB)"), &mut device.selected_gain, gains.as_slice(), -1);
                    });

                    ui.separator();
                });
            }
            if state.selected_device != selected_device {
                state.selected_device = selected_device;
            }
        });

        // Show log
        ui.tree_node(im_str!("Log")).build(|| {
            ui.child_frame(im_str!("_log_view"), (-10.0, 200.0)).
                show_borders(true).
                always_show_vertical_scroll_bar(true).
                build(||{
                    for line in &state.log {
                        ui.text(im_str!("{}", line));
                    }
                });
        });
    }
}

fn render_full_view(ui: &Ui, state: &Arc<Mutex<State>>) {
    if ui.collapsing_header(im_str!("Full view")).build() {
        let points = &state.lock().unwrap().data;
        let width = ui.get_window_size().0 - 15.0;
        ui.
            child_frame(im_str!("_chart_frame"), (0.0, 0.0)).
            show_borders(true).
            build(||{
                ui.push_item_width(0.0);
                ui.plot_lines(im_str!("##chart_full"), &points[..]).
                    graph_size(ui.get_item_rect_size()).
                    scale_min(0.0).
                    build();
            });
    }
}
