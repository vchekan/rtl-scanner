#![allow(non_snake_case)]
#[macro_use]
extern crate qml;
extern crate rtlsdr;
extern crate libc;
extern crate num;

mod fftw;
mod scanner;
mod rtl_import;

use qml::*;
use rtlsdr::RTLSDRDevice;
use fftw::Plan;
use rtl_import::*;
use std::cmp::Ordering;

const SAMPLERATE: u32 = 2e6 as u32;
const BANDWIDTH: u32 = 1e6 as u32;
// TODO: make dwell selectable
const DWELL_MS: u32 = 16;

pub struct Scanner {
    device: Option<RTLSDRDevice>,
    width: i32,
    height: i32
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

fn rescale(width: i32, height: i32, data: &Vec<f64>) -> Vec<f64> {
    let mut res = Vec::with_capacity(width as usize);
    //let max = data.iter().cloned().fold(0./0., f64::max);

    let samples_per_pixel = data.len() as f32 / width as f32;
    let mut max = ::std::f64::MIN;
    for i in 0..width {
        // TODO: averaging shrinks dynamic range. Maybe do percentiles and adjust color intensity accordingly?
        let start_sample = (i as f32 * samples_per_pixel).round() as usize;
        let end_sample = ((i+1) as f32 * samples_per_pixel).round() as usize;
        let avg: f64 = data.as_slice()[start_sample..end_sample].iter().sum::<f64>() / samples_per_pixel as f64;
        res.push(avg);
        if avg > max
            {max = avg;}
    }

    for i in 0..res.len() {res[i] = res[i] / max * height as f64}

    res
}

// TODO: handle device calls more intelligently than just unwrap(). If device is removed from usb
// and function call fail, it would cause panic.
impl QScanner {
    pub fn InitHarware(&mut self) -> Option<&QVariant> {
        self.threaded(|s| {
            // TODO: send error message if failed and keep retrying
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

    pub fn start(&mut self, from: i32, to: i32) -> Option<&QVariant> {
        self.threaded(move |s| {
            s.status("scanning...".to_string());

            let step = BANDWIDTH / 2;
            let start = from as u32 - BANDWIDTH;
            let end = to as u32 + BANDWIDTH * 2;
            // TODO: align to 512
            let sample_count = (DWELL_MS * SAMPLERATE) / 1000;
            let buffer_size = calculate_aligned_buffer_size(sample_count);
            println!("Buffer size {} bytes, {} samples", buffer_size, sample_count);

            let fftPlan = Plan::new(sample_count as usize);

            let driver = s.device.as_ref().unwrap();
            driver.set_sample_rate(SAMPLERATE).unwrap();
            driver.set_tuner_bandwidth(BANDWIDTH).unwrap();
            driver.reset_buffer().unwrap();

            let input = fftPlan.get_input();
            let output = fftPlan.get_output();

            println!("Scanning from {} to {}", from, end);
            let mut freq: u32 = start;
            while freq <= end {
                driver.set_center_freq(freq).unwrap();
                // TODO: add borrowed buffer override to rtlsdr driver
                let buffer = driver.read_sync(buffer_size as usize).unwrap();
                freq += step;

                rtl_import(&buffer, buffer.len(), input);
                fftPlan.execute();

                let data = complex_to_abs(output);
                let rescaled = rescale(s.width, s.height, &data);
                let data_qv = rescaled.iter().map(|&x| x.into()).collect::<Vec<_>>();
                s.plot(data_qv.into());

                break;
            }

            s.status("Scanning finished".to_string());
        });
        None
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Option<&QVariant> {
        self.width = width;
        self.height = height;
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
        fn start(from: i32, to: i32);
        fn resize(width: i32, height: i32);
    properties:
});

fn startUi() {
    let mut engine = QmlEngine::new();
    let qscanner = QScanner::new(Scanner {device: None, width: 0, height: 0});
    engine.set_and_store_property("scanner", qscanner.get_qobj());
    engine.load_file("src/scanner.qml");
    engine.exec();
}

fn main() {
    startUi();
    // TODO: implement index
}

fn print_info(idx: i32) {
    let res = rtlsdr::get_device_usb_strings(idx).unwrap();
    println!("  Manufacturer: {}", res.manufacturer);
    println!("  Product:      {}", res.product);
    println!("  Serial:       {}", res.serial);

    let name = rtlsdr::get_device_name(idx);
    println!("  Name: {}", name);
}