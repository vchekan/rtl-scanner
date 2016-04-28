extern crate rtlsdr;
extern crate qmlrs;

const SAMPLERATE: u32 = 2e6 as u32;
const BANDWIDTH: u32 = 1e6 as u32;


fn main() {
    let scanner_qml = include_str!("scanner.qml");
    let mut engine = qmlrs::Engine::new();
    engine.load_data(scanner_qml);
    engine.exec();

    /*
    // TODO: implement index
    let idx = 0;

    //print_info(idx);
    let mut dev = rtlsdr::open(idx).unwrap();
    rtl_configure(&mut dev);

    //dev.close().unwrap();
   */
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

    dev.set_tuner_bandwidth(BANDWIDTH);//.unwrap();
    println!("  Set bandwidth {}Mhz", BANDWIDTH/1000);

    //dev.reset_buffer().unwrap();
}

fn read(dev: &rtlsdr::RTLSDRDevice) {

}