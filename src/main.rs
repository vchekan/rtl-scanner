#![allow(non_snake_case)]
#[macro_use]
extern crate qml;
use rtlsdr;



mod fftw;
mod rtl_import;
mod dsp;
mod iterators;
mod charts;
mod spectrum;

use qml::*;
use rtlsdr::RTLSDRDevice;
use crate::fftw::Plan;
use crate::rtl_import::*;
use std::cmp::Ordering;
use num::complex::*;
use crate::charts::*;
use crate::iterators::*;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

use std::thread;
use std::time::Duration;

const SAMPLERATE: usize = 2e6 as usize;
const BANDWIDTH: usize = 1e6 as usize;
// TODO: make dwell selectable
const DWELL_MS: usize = 16;

static dump_data: bool = true;


//#[derive(Debug)] TODO: immplement inside in RTLSDRDriver
pub struct Scanner {
    device: Option<RTLSDRDevice>,
    width: i32,
    height: i32,
    samples: Arc<Mutex<spectrum::Samples>>
}

impl fmt::Debug for Scanner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "Scanner {{{}x{} {:?}}}", self.width, self.height, self.samples)
        write!(f, "Scanner {{{}x{} }}", self.width, self.height)
    }
}

impl Scanner {
    fn new(f_sampling: usize, start: u32, end: u32, dwell_ms: usize, bandwidth: usize) -> Scanner {
        Scanner {
            device: None,
            height: 0,
            width: 0,
            samples: Arc::new(Mutex::new(spectrum::Samples::new(f_sampling, start as usize, end as usize, dwell_ms, bandwidth)))
        }
    }
}

fn cmp_f64(_self: &f64, other: &f64) -> Ordering {
    _self.partial_cmp(other).unwrap_or(Ordering::Less)
}

pub fn calculate_aligned_buffer_size(samples: usize) -> usize {
    // a sample is a complex byte, thus 2 bytes per sample
    let bytes = samples * 2;
    // TODO: align to ^2 because FFTW works the fastest than
    return bytes + bytes % 512;
}


// TODO: handle device calls more intelligently than just unwrap(). If device is removed from usb
// and function call fail, it would cause panic.
impl QScanner {
    pub fn InitHarware(&mut self) -> Option<&QVariant> {
        self.threaded(|s| {
            // TODO: send error message if failed and keep retrying
            // TODO: implement index
            let idx = 0;
            let mut dev = rtlsdr::open(idx).unwrap();
            print_info(idx);
            let res = rtlsdr::get_device_usb_strings(idx).unwrap();

            // Show device name
            s.showRtlProduct(res.product);

            // show gains
            let gains: Vec<i32> = dev.get_tuner_gains().unwrap();
            println!("  Available gains: {:?}", &gains);
            let qv_gains = gains.iter().map(|&x| x.into()).collect::<Vec<_>>();
            s.gains(qv_gains.into());
            dev.set_agc_mode(true).unwrap();

            s.device = Some(dev);
        });
        None
    }

    pub fn start(&mut self, from: f64, to: f64) -> Option<&QVariant> {
        self.threaded(move |s| {
            s.status("scanning...".to_string());

            let step = BANDWIDTH / 2;
            let start = (from * 1e6) as usize - BANDWIDTH;
            let end = (to * 1e6) as usize + BANDWIDTH * 2;

            //s.samples = Arc::new(Mutex::new(spectrum::Samples::new(SAMPLERATE, start, end, DWELL_MS, BANDWIDTH)));

            // TODO: align to 512
            let sample_count = (DWELL_MS * SAMPLERATE) / 1000;
            let buffer_size = calculate_aligned_buffer_size(sample_count);
            println!("Buffer size {} bytes, {} samples", buffer_size, sample_count);

            let fftPlan = Plan::new(sample_count as usize);

            {
                let driver = s.device.as_mut().unwrap();
                driver.set_sample_rate(SAMPLERATE as u32).unwrap();
                driver.set_tuner_bandwidth(BANDWIDTH as u32).unwrap();
                driver.reset_buffer().unwrap();
            }

            let input = fftPlan.get_input();
            let output: &[f64] = fftPlan.get_output();

            /*
            let mut file = match dump_data {true => Some(BufWriter::new(File::create("./data/raw.mat").unwrap())), false => None};
            if dump_data {
                //let mut file = BufWriter::new(File::create("raw.mat").unwrap());
                file.as_mut().unwrap().write_all(b"# Created by rtl-scanner\n");
                file.as_mut().unwrap().write_all(b"# name: raw_bytes\n");
                file.as_mut().unwrap().write_all(b"# type: matrix\n");
                write!(file.as_mut().unwrap(), "# rows: {}\n", ((end - start) as f64 / step as f64).ceil());
                write!(file.as_mut().unwrap(), "# columns: {}\n", buffer_size );
            }
            */

            //
            // TODO: think, if it is possible to do frequencies in rational space and not in f64.
            // Maybe bandwidth could be a basic unit of measure?
            //
            // TODO: research delay needed to avoid empty buffer at the start after change frequency
            //

            println!("Scanning from {} to {}", from, end);
            let mut freq: usize = start;
            let mut i = 0;

            //print!("Estimated lines: {} {}\n", (end - start) as f64/ step as f64, ((end - start) as f64 / step as f64).ceil());

            while freq <= end {
                let buffer: Vec<u8>;
                {
                    let driver = s.device.as_mut().unwrap();
                    driver.set_center_freq(freq as u32).unwrap();
                    // TODO: add borrowed buffer override to rtlsdr driver
                    buffer = driver.read_sync(buffer_size as usize).unwrap();
                }

                /*if file.is_some() {
                    let f = file.as_mut().unwrap();
                    for b in &buffer {
                        write!(f, "{} ", b);
                    }
                    write!(f, "\n");
                }*/


                freq += step;
                if i % 10 == 0 {
                    println!("> {}", freq as f64/1e6);
                }
                i += 1;

                rtl_import(&buffer, buffer.len(), input);
                fftPlan.execute();

                // http://www.fftw.org/doc/The-1d-Discrete-Fourier-Transform-_0028DFT_0029.html#The-1d-Discrete-Fourier-Transform-_0028DFT_0029
                // Note also that we use the standard “in-order” output ordering—the k-th output corresponds to the frequency
                // k/n (or k/T, where T is your total sampling period). For those who like to think in terms of positive and
                // negative frequencies, this means that the positive frequencies are stored in the first half of the output
                // and the negative frequencies are stored in backwards order in the second half of the output.
                // (The frequency -k/n is the same as the frequency (n-k)/n.)
                //
                // Or just numpy implementation:
                // https://github.com/numpy/numpy/blob/v1.12.0/numpy/fft/helper.py#L74

                // TODO: smooth 0th frequency
                let dft_out_ordered = output[output.len()/2..].iter().chain(output[..output.len()/2].iter()).cloned().tuples();
                let complex_dft = dft_out_ordered.
                    map(|(re, im)| Complex64::new(re, im)).
                    // TODO: do not collect but keep propagating Iterator into ::psd
                    collect::<Vec<_>>();

                println!("output[]: {:?}", output[0..100].iter());

                let psd = dsp::psd(&complex_dft);

                let _fft_step = 1.0 / (DWELL_MS as f64 / 1000.0);
                let mut samples = s.samples.lock().unwrap();
                for c in psd.into_iter() {
                    samples.samples.push(c);
                }
            }

            s.status("Scanning finished".to_string());
            s.refresh();
        });
        None
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Option<&QVariant> {
        self.width = width;
        self.height = height;
        self.refresh();
        None
    }

    fn refresh(&self) {
        let samples = self.samples.lock().unwrap();
        if samples.samples.len() > 0 {
            let rescaled = rescale(self.width, self.height, &samples.samples);
            let data_qv = rescaled.into_iter().map(|x| x.into()).collect::<Vec<_>>();
            self.plot(data_qv.into());
        }
    }
}

pub struct Logic;

impl QLogic {
    pub fn downloadPage(&mut self, url: String) -> Option<&QVariant>{
        self.threaded(|s| {
            thread::sleep(Duration::from_secs(2));;
            s.pageDownloaded(url);
        });
        None
    }
}

Q_OBJECT!{
pub Logic as QLogic {
    signals:
        fn pageDownloaded(response: String);
    slots:
        fn downloadPage(url: String);
    properties:
}}

Q_OBJECT!(
pub Scanner as QScanner {
    signals:
        fn showRtlProduct(product: String);
        fn gains(gainList: QVariantList);
        fn status(text: String);
        fn plot(data: QVariantList);
    slots:
        fn InitHarware();
        fn start(from: f64, to: f64);
        fn resize(width: i32, height: i32);
    properties:
});

fn main() {
    let mut engine = QmlEngine::new();
    let scanner = Scanner::new(SAMPLERATE, 100_000_000, 200_000_000, DWELL_MS, BANDWIDTH);
    let qscanner = QScanner::new(scanner);
    engine.set_and_store_property("scanner", qscanner.get_qobj());
    engine.load_file("src/scanner.qml");
    engine.exec();

    println!("done");
    std::process::exit(0);
}

fn print_info(idx: i32) {
    let res = rtlsdr::get_device_usb_strings(idx).unwrap();
    println!("  Manufacturer: {}", res.manufacturer);
    println!("  Product:      {}", res.product);
    println!("  Serial:       {}", res.serial);

    let name = rtlsdr::get_device_name(idx);
    println!("  Name: {}", name);
}