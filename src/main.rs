mod fftw;
mod dsp;
mod iterators;
mod charts;
mod samples;
mod scanner;
mod ui;

use rtlsdr::{self, RTLSDRError, RTLSDRDevice, USBStrings};
use crate::fftw::Plan;
use crate::charts::*;
use crate::iterators::*;
use std::{
    fmt,
    fs::File,
    io::{prelude::*, BufWriter},
    error::Error,
    iter::Iterator,
    thread,
    time::Duration,
    cmp::Ordering
};
use log::{debug, error, info};
use structopt::StructOpt;

use simplelog::*;
use crate::scanner::{Scanner, ScannerStatus};
use std::process::exit;
use std::thread::Thread;

use druid::Data;

const SAMPLERATE: usize = 2e6 as usize;
const BANDWIDTH: usize = 1e6 as usize;
// TODO: make dwell selectable
const DWELL_MS: usize = 16;

#[derive(Debug, Clone, Data, PartialEq)]
pub struct Device{
    pub name: String,
    pub manufacturer: String,
    pub product: String,
    pub serial: String
}

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(long)]
    dump: bool,
    #[structopt(long)]
    list: bool,
    #[structopt(long)]
    device: Option<String>,
}

fn main() {
    SimpleLogger::init(LevelFilter::Debug, Config::default());

    #[cfg(feature = "imgui")] crate::wininit::main::main();
    
    #[cfg(feature = "druid-ui")] crate::ui::druid::main();

/*
    let opts = Cli::from_args();
    debug!("Opts: {:?}", opts);
    if opts.list {
        for i in 0..rtlsdr::get_device_count() {
            let name = rtlsdr::get_device_name(i);
            let usb_string = rtlsdr::get_device_usb_strings(i).
                map(|s|format!("{:?}", s)).
                unwrap_or("-error-".to_string());
            println!("Name: \"{}\"; {}", name, usb_string);
        }
        return;
    }

    let idx = choose_device(&opts.device);
    */

    /*
    let mut scanner = Scanner::new(idx, SAMPLERATE, 100_000_000, 120_000_000, DWELL_MS, BANDWIDTH, opts.dump);
    let from_scanner = scanner.start();
    while let Ok(msg) = from_scanner.recv() {
        match msg {
            ScannerStatus::Info(msg) => info!("{}", msg),
            ScannerStatus::Error(msg) => error!("{}", msg),
            ScannerStatus::Data(data) => {},
        }
    }
    */
}

fn choose_device(device: &Option<String>) -> i32 {
    let count = rtlsdr::get_device_count();
    debug!("Detected {} device(s)", count);
    match (count, &device) {
        (0,_) => {
            eprintln!("No devices detected");
            exit(1)
        },
        // Single device present, no filter provider
        (1, None) => {
            debug!("Selected device: {:?}", rtlsdr::get_device_name(0));
            0
        },
        // Multiple device present, no filter provided
        // Choose 1st one, but write warning
        (_, None) => {
            eprintln!("Detected multiple devices, will use the 1st one. Use --device <pattern> to specify device. {:?}",
                      rtlsdr::get_device_name(0));
            0
        },
        // Have device pattern
        (_,Some(device)) => {
            let mut device = device.clone();
            device.make_ascii_uppercase();
            for i in 0..count {
                let mut name = rtlsdr::get_device_name(i);
                name.make_ascii_uppercase();
                if name.contains(&device) {
                    debug!("Matched device pattern '{}' to device: '{}'", device, name);
                    return i;
                }
            }
            // No matches
            eprintln!("No matches found. Use --list to see devices present.");
            exit(1)
        }
    }
}

pub fn list_devices() -> Vec<Device> {
    let count = rtlsdr::get_device_count();
    let mut devices = Vec::with_capacity(count as usize);
    for i in 0..count {
        let name = rtlsdr::get_device_name(i);
        let usb = rtlsdr::get_device_usb_strings(i).ok();
        let device = match usb {
            Some(usb) => Device {name, manufacturer: usb.manufacturer, product: usb.product, serial: usb.serial},
            None => Device {name, manufacturer: String::new(), product: String::new(), serial: String::new()},
        };
        devices.push(device);
    }
    devices
}