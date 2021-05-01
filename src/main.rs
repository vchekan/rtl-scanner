mod fftw;
mod dsp;
mod iterators;
mod charts;
mod samples;
mod scanner;
mod ui;

use rtlsdr::{self, RTLSDRError, RTLSDRDevice};
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

const SAMPLERATE: usize = 2e6 as usize;
const BANDWIDTH: usize = 1e6 as usize;
// TODO: make dwell selectable
const DWELL_MS: usize = 16;

/*
fn cmp_f64(_self: &f64, other: &f64) -> Ordering {
    _self.partial_cmp(other).unwrap_or(Ordering::Less)
}
*/

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
    crate::ui::main::main();

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
