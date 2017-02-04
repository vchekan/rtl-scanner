#![allow(non_snake_case)]
#[macro_use]
extern crate qml;
extern crate rtlsdr;
extern crate libc;
extern crate num;

mod fftw;
mod scanner;
mod rtl_import;
mod dsp;
mod iterators;
mod charts;

use qml::*;
use rtlsdr::RTLSDRDevice;
use fftw::Plan;
use rtl_import::*;
use std::cmp::Ordering;
use num::complex::*;
use charts::*;
use iterators::*;
use std::fmt;

const SAMPLERATE: u32 = 2e6 as u32;
const BANDWIDTH: u32 = 1e6 as u32;
// TODO: make dwell selectable
const DWELL_MS: u32 = 16;

#[derive(Debug)]
struct Bucket {
    freq_min: f64,
    freq_max: f64,
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    freq: f64,
    psd: f64
    //sum: f64,
    //count: u32,
    //min: i32,
    //max: i32
}

#[derive(Debug)]
struct Spectrum {
    freq_start: u32,
    freq_end: u32,
    buckets: Vec<Bucket>,
}

//#[derive(Debug)] TODO: immplement inside in RTLSDRDriver
pub struct Scanner {
    device: Option<RTLSDRDevice>,
    width: i32,
    height: i32,
    snapshot: Vec<f64>,
    spectrum: Spectrum,
}

impl fmt::Debug for Scanner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Scanner {{{}x{} {:?}}}", self.width, self.height, self.spectrum)
    }
}

impl Spectrum {
    fn new(start: u32, end: u32, buckets: usize) -> Spectrum {
        let freq_step = (end - start) as f64 / buckets as f64;
        let mut freq = start as f64;
        let mut vec = Vec::with_capacity(buckets);
        for i in 0 .. buckets {
            vec.push(Bucket{freq_min: freq, freq_max: freq+freq_step, entries: Vec::new()});
            freq += freq_step;
        }

        Spectrum {
            buckets: vec,
            freq_start: start,
            freq_end: end
        }
    }

    fn set_range(&mut self, start: u32, end: u32) {
        // TODO: this is almost cpy-paste from new()
        let len = self.buckets.len();
        let freq_step = (end - start) as f64 / len as f64;
        self.freq_start = start;
        self.freq_end = end;
        self.buckets.clear();
        let mut freq = start as f64;
        for i in 0 .. len {
            self.buckets.push(Bucket{freq_min: freq, freq_max: freq+freq_step, entries: Vec::new()});
            freq += freq_step;
        }
    }
}

impl Scanner {
    fn new(start: u32, end: u32, buckets: usize) -> Scanner {
        Scanner {
            device: None,
            height: 0,
            width: 0,
            snapshot: Vec::new(),
            spectrum: Spectrum::new(start, end, buckets)
        }
    }
}

fn cmp_f64(_self: &f64, other: &f64) -> Ordering {
    _self.partial_cmp(other).unwrap_or(Ordering::Less)
}

pub fn calculate_aligned_buffer_size(samples: u32) -> u32 {
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
            let dev = rtlsdr::open(idx).unwrap();
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
            let start = (from * 1e6) as u32 - BANDWIDTH;
            let end = (to * 1e6) as u32 + BANDWIDTH * 2;

            s.spectrum.set_range(start, end);

            // TODO: align to 512
            let sample_count = (DWELL_MS * SAMPLERATE) / 1000;
            let buffer_size = calculate_aligned_buffer_size(sample_count);
            println!("Buffer size {} bytes, {} samples", buffer_size, sample_count);

            let fftPlan = Plan::new(sample_count as usize);

            {
                let driver = s.device.as_ref().unwrap();
                driver.set_sample_rate(SAMPLERATE).unwrap();
                driver.set_tuner_bandwidth(BANDWIDTH).unwrap();
                driver.reset_buffer().unwrap();
            }

            let input = fftPlan.get_input();
            let output: &[f64] = fftPlan.get_output();

            //
            // TODO: think, if it is possible to do frequencies in rational space and not in f64.
            // Maybe bandwidth could be a basic unit of measure?
            //
            // TODO: smooth central frequency
            //

            println!("Scanning from {} to {}", from, end);
            let mut freq: u32 = start;
            while freq <= end {
                let buffer: Vec<u8>;
                {
                    let driver = s.device.as_ref().unwrap();
                    driver.set_center_freq(freq).unwrap();
                    // TODO: add borrowed buffer override to rtlsdr driver
                    buffer = driver.read_sync(buffer_size as usize).unwrap();
                }
                freq += step;

                rtl_import(&buffer, buffer.len(), input);
                fftPlan.execute();

                // http://www.fftw.org/doc/The-1d-Discrete-Fourier-Transform-_0028DFT_0029.html#The-1d-Discrete-Fourier-Transform-_0028DFT_0029
                // Note also that we use the standard “in-order” output ordering—the k-th output corresponds to the frequency
                // k/n (or k/T, where T is your total sampling period). For those who like to think in terms of positive and
                // negative frequencies, this means that the positive frequencies are stored in the first half of the output
                // and the negative frequencies are stored in backwards order in the second half of the output.
                // (The frequency -k/n is the same as the frequency (n-k)/n.)

                // TODO: test 0th frequency
                let dft_out_ordered = output[output.len()/2..].iter().chain(output[..output.len()/2].iter()).cloned().tuples();
                let complex_dft = dft_out_ordered.
                    map(|(re, im)| Complex64::new(re, im)).
                    // TODO: do not collect but keep propagating Iterator into ::psd
                    collect::<Vec<_>>();
                let psd = dsp::psd(&complex_dft);

                let fft_step = 1.0 / (DWELL_MS as f64 / 1000.0);
                let bucket_step = (s.spectrum.freq_end - s.spectrum.freq_start) as f64 / s.spectrum.buckets.len() as f64;
                let mut f: f64 = (freq - BANDWIDTH) as f64 / 2.0;
                let mut bucket_freq = s.spectrum.freq_start as f64 + bucket_step;
                let mut bucketIndex: usize = ((freq - s.spectrum.freq_start) as f64 / bucket_step).floor() as usize;
                println!("freq: {}, freq_start: {}, bucketIndex: {}", freq, s.spectrum.freq_start, bucketIndex);
                println!("scanner: {:?}", s as &mut Scanner);
                for power in &psd {
                    let ref mut bucket = s.spectrum.buckets[bucketIndex];
                    bucket.entries.push(Entry {
                        freq: f,
                        psd: *power
                    });
                    f += fft_step;
                    if f > bucket_freq {
                        bucket_freq += bucket_step;
                    }
                };

                let rescaled = rescale(s.width, s.height, &psd);
                let data_qv = rescaled.iter().map(|&x| x.into()).collect::<Vec<_>>();

                s.plot(data_qv.into());

                s.snapshot = psd;

                break;
            }

            s.status("Scanning finished".to_string());
        });
        None
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Option<&QVariant> {
        self.width = width;
        self.height = height;

        if self.snapshot.len() > 0 {
            let rescaled = rescale(self.width, self.height, &self.snapshot);
            let data_qv = rescaled.iter().map(|&x| x.into()).collect::<Vec<_>>();
            self.plot(data_qv.into());
        }

        None
    }
}

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

fn startUi() {
    let mut engine = QmlEngine::new();
    let qscanner = QScanner::new(Scanner::new(100_000_000, 200_000_000, 3000));
    engine.set_and_store_property("scanner", qscanner.get_qobj());
    engine.load_file("src/scanner.qml");
    engine.exec();
}

fn main() {
    startUi();
}

fn print_info(idx: i32) {
    let res = rtlsdr::get_device_usb_strings(idx).unwrap();
    println!("  Manufacturer: {}", res.manufacturer);
    println!("  Product:      {}", res.product);
    println!("  Serial:       {}", res.serial);

    let name = rtlsdr::get_device_name(idx);
    println!("  Name: {}", name);
}