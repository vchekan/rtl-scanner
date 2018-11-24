use rtlsdr;

mod fftw;
mod rtl_import;
mod dsp;
mod iterators;
mod charts;
mod samples;
mod support_gfx;
mod scanner;
mod gui;

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
use crate::gui::{render};


#[macro_use] extern crate log;

use crate::gui::State;
use rtlsdr::RTLSDRError;
use crate::gui::Device;
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
