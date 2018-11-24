use rtlsdr::USBStrings;
use std::collections::VecDeque;
use imgui::ImString;
use rtlsdr::RTLSDRError;
use crate::scanner::{Scanner, ScannerStatus};
use futures::sync::mpsc::UnboundedReceiver;
use std::sync::{Arc, Mutex};

const LOG_LEN: usize = 100;

pub(crate) struct State {
    pub show_log: bool,
    // TODO: do colors for errors
    pub log: VecDeque<String>,
    pub devices: Vec<Device>,
    pub selected_device: usize,
    pub scan_from: u32,
    pub scan_to: u32,
    pub is_running: bool,
    pub scanner_cmd: Option<Arc<Mutex<VecDeque<ScannerStatus>>>>,
    pub data: Vec<f32>,
}

pub(crate) struct Device {
    pub name: String,
    pub usb_description: USBStrings,
    pub gains: Vec<ImString>,
    pub selected_gain: i32,
    pub tuner_type: String,
}

impl State {
    pub fn new() -> Self {
        State {
            show_log: false,
            log: VecDeque::with_capacity(100),
            devices: vec![],
            selected_device: 0,
            scan_from: 60e6 as u32,
            scan_to: 1700e6 as u32,
            is_running: false,
            scanner_cmd: None,
            data: vec![],
        }
    }

    pub fn append_log(&mut self, str: String) {
        while self.log.len() > LOG_LEN - 1 {
            self.log.pop_front();
        }
        self.log.push_back(str);
    }
}

impl Device {
    pub fn probe(idx: i32) -> Result<Self, RTLSDRError> {
        let mut dev = rtlsdr::open(idx)?;
        let name = rtlsdr::get_device_name(idx);
        let usb_description = rtlsdr::get_device_usb_strings(idx)?;
        let gains = dev.get_tuner_gains()?.iter().
            // Gain is reported in 10th of Db
            map(|gain| ImString::new(format!("{}", (*gain as f32)/10.0))).collect::<Vec<_>>();
        let (_, tuner_type) = dev.get_tuner_type();
        Ok(Device{ name, usb_description, gains, selected_gain: 0, tuner_type })
    }
}