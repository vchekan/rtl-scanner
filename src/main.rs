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
use std::time::Duration;
use rtlsdr::RTLSDRDevice;
use fftw::Plan;
use rtl_import::*;

const SAMPLERATE: u32 = 2e6 as u32;
const BANDWIDTH: u32 = 1e6 as u32;
// TODO: make dwell selectable
const DWELL_MS: u32 = 16;

pub struct Scanner {
    device: Option<RTLSDRDevice>,
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
            let idx = 0;
            let mut dev = rtlsdr::open(idx).unwrap();
            let res = rtlsdr::get_device_usb_strings(idx).unwrap();

            // Show device name
            s.showRtlProduct(res.product);

            // show gains
            let gains: Vec<i32> = dev.get_tuner_gains().unwrap();
            println!("  Available gains: {:?}", &gains);
            let qv_gains = gains.iter().map(|&x| x.into()).collect::<Vec<_>>();
            s.gains(qv_gains.into());
            dev.set_agc_mode(true);

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
            driver.set_tuner_bandwidth(BANDWIDTH);
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
                let data_qv = data.iter().map(|&x| x.into()).collect::<Vec<_>>();
                s.plot(data_qv.into());

                break;
            }

            s.status("Scanning finished".to_string());
        });
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
    properties:
});

fn startUi() {
    let mut engine = QmlEngine::new();
    let qscanner = QScanner::new(Scanner {device: None});
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

fn rtl_configure(dev: &mut rtlsdr::RTLSDRDevice) {
    dev.set_tuner_gain_mode(true).unwrap();

    let gains = dev.get_tuner_gains().unwrap();
    println!("  Available gains: {:?}", &gains);
    let gain: i32 = *gains.last().unwrap();

    dev.set_tuner_gain(gain).unwrap();

    let gain = dev.get_tuner_gain();
    println!("  Current gain: {}dB", (gain as f64)/10.0f64);

    println!("  Setting sample rate to {}kHz", (SAMPLERATE as f64)/1000.0f64);
    dev.set_sample_rate(SAMPLERATE).unwrap();

    dev.set_tuner_bandwidth(BANDWIDTH).unwrap();
    println!("  Set bandwidth {}Mhz", BANDWIDTH/1000);
}
