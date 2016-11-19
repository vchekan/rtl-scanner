#![allow(non_snake_case)]
#[macro_use]
extern crate qml;
extern crate rtlsdr;

use qml::*;
use std::time::Duration;
use rtlsdr::RTLSDRDevice;

const SAMPLERATE: u32 = 2e6 as u32;
const BANDWIDTH: u32 = 1e6 as u32;

pub struct Scanner {
    device: Option<RTLSDRDevice>,
}

impl QScanner {
    pub fn InitHarware(&mut self) -> Option<&QVariant> {
        self.threaded(|s| {
            // TODO: send error message if failed and keep retrying
            let idx = 0;
            s.device = Some(rtlsdr::open(idx).unwrap());
            let res = rtlsdr::get_device_usb_strings(idx).unwrap();

            // Show device name
            s.rtlProduct(res.product);

            // show gains
            let gains: Vec<i32> = s.device.as_mut().unwrap().get_tuner_gains().unwrap();
            println!("  Available gains: {:?}", &gains);
            let qv_gains = gains.iter().map(|&x| x.into()).collect::<Vec<_>>();
            s.gains(qv_gains.into());
        });
        None
    }

    pub fn start(&mut self, from: i32, to: i32) -> Option<&QVariant> {
        self.threaded(|s| {
            s.status("scanning".to_string());
        });
        None
    }
}

Q_OBJECT!(
pub Scanner as QScanner {
    signals:
        fn rtlProduct(product: String);
        fn gains(gainList: QVariantList);
        fn dataReady(data: QVariantList);
        fn status(text: String);
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

    //println!("  Setting gain to second option {}dB",
    //(gains[1] as f64)/10.0f64);
    dev.set_tuner_gain(gain).unwrap();

    let gain = dev.get_tuner_gain();
    println!("  Current gain: {}dB", (gain as f64)/10.0f64);

    println!("  Setting sample rate to {}kHz", (SAMPLERATE as f64)/1000.0f64);
    dev.set_sample_rate(SAMPLERATE).unwrap();

    dev.set_tuner_bandwidth(BANDWIDTH).unwrap();
    println!("  Set bandwidth {}Mhz", BANDWIDTH/1000);

    //dev.reset_buffer().unwrap();
}
