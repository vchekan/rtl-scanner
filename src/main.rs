use rtlsdr;

mod fftw;
mod rtl_import;
mod dsp;
mod iterators;
mod charts;
mod samples;
mod support_gfx;
mod state;
mod scanner;

use rtlsdr::RTLSDRDevice;
use crate::fftw::Plan;
use crate::rtl_import::*;
use num::complex::*;
use crate::charts::*;
use crate::iterators::*;
use std::{
    fmt,
    fs::File,
    io::{prelude::*, BufWriter},
    sync::{Arc, Mutex},
    error::Error,
    iter::Iterator,
    thread,
    time::Duration,
    cmp::Ordering
};


#[macro_use] extern crate log;

use imgui::*;

use crate::state::State;
use rtlsdr::RTLSDRError;
use crate::state::Device;
use simplelog::*;
use crate::scanner::{Scanner, ScannerStatus};

const SAMPLERATE: usize = 2e6 as usize;
const BANDWIDTH: usize = 1e6 as usize;
// TODO: make dwell selectable
const DWELL_MS: usize = 16;
const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

static dump_data: bool = true;

fn cmp_f64(_self: &f64, other: &f64) -> Ordering {
    _self.partial_cmp(other).unwrap_or(Ordering::Less)
}

fn main() {
    CombinedLogger::init(vec![TermLogger::new(LevelFilter::Debug, Config::default()).unwrap()]);
    let state = Arc::new(Mutex::new(State::new()));
    start_device_loop(state.clone());

    support_gfx::run("RTL Scanner".to_owned(), CLEAR_COLOR, render, state);
}

fn start_device_loop(state: Arc<Mutex<State>>) {
    thread::spawn(move || {
        loop {
            if let Err(err) = device_loop(state.clone()) {
                state.lock().unwrap().append_log(err.to_string());
                // Prevent error storm
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

fn device_loop(state: Arc<Mutex<State>>) -> Result<(),RTLSDRError> {
    // TODO: dynamically re-scan devices
    // TODO: detect frequency ranges
    // TODO: detect direct sampling

    let count = rtlsdr::get_device_count();
    { state.lock().unwrap().append_log(format!("Found {} device(s)", count))}

    let devices = (0..count).map(Device::probe).
        map(|e| {
            match e {
                Ok(dev) => Some(dev),
                Err(err) => {
                    state.lock().unwrap().append_log(err.to_string());
                    None
                }
            }
        }).flatten().collect::<Vec<_>>();

    state.lock().unwrap().devices = devices;

    loop {
        std::thread::sleep(Duration::from_secs(1));
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

fn render(ui: &Ui, state: &mut Arc<Mutex<State>>) -> bool {
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
            render_full_view(&ui, &state);
            ui.separator();

            render_scan(&ui, &state);
            ui.separator();

            render_settings(&ui, &state);
        });
    });


    true
}

fn render_full_view(ui: &Ui, state: &Arc<Mutex<State>>) {
    if ui.collapsing_header(im_str!("Full view")).build() {
        let points = &state.lock().unwrap().data;
        let width = ui.get_window_size().0 - 15.0;
        ui.plot_lines(im_str!("##chart_full"), &points[..]).
            graph_size((width, 200.0)).
            build();
    }
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
            ui.child_frame(im_str!("_log_view"), (-10.0, 0.0)).
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