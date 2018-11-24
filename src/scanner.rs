use rtlsdr::RTLSDRDevice;
use std::sync::{Arc, Mutex};
use crate::samples;
use crate::fftw::Plan;
use crate::dsp;
use crate::rtl_import::rtl_import;
use crate::charts::rescale;
use crate::iterators::TuplesImpl;
use std::thread;
use num::complex::*;
use futures::{
    prelude::*,
    sync::mpsc::{channel, UnboundedSender, UnboundedReceiver}
};
use log::{error, info, debug};
use crate::samples::Samples;
use futures::sync::BiLock;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Scanner {
    device_index: i32,
    width: i32,
    height: i32,
    samples: Arc<Mutex<samples::Samples>>,
    bandwidth: usize,
    dwell_ms: usize,
    samplerate: usize,
    from: u32,
    to: u32,
}

pub enum ScannerStatus {
    Info(String),
    Error(String),
    Data(Vec<f64>),
    Complete,
}

impl Scanner {
    pub fn new(device_index: i32, samplerate: usize, from: u32, to: u32, dwell_ms: usize, bandwidth: usize) -> Scanner {
        let device = rtlsdr::open(device_index);
        Scanner {
            device_index,
            height: 0,
            width: 0,
            samples: Arc::new(Mutex::new(samples::Samples::new(samplerate, from as usize, to as usize, dwell_ms, bandwidth))),
            bandwidth,
            dwell_ms,
            samplerate,
            from,
            to
        }
    }


    // TODO: handle device calls more intelligently than just unwrap(). If device is removed from usb
    // and function call fail, it would cause panic.
    pub fn start(mut self) -> Arc<Mutex<VecDeque<ScannerStatus>>> {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let queue2 = queue.clone();
        thread::spawn(move || {
            if let Err(err) = self.scan(&queue2) {
                let mut queue2 = queue2.lock().unwrap();
                queue2.push_back(ScannerStatus::Error(err.to_string()));
            }
        });
        queue
    }

    fn scan(&mut self, channel: &Arc<Mutex<VecDeque<ScannerStatus>>>) -> Result<(), rtlsdr::RTLSDRError> {
        let mut driver = rtlsdr::open(self.device_index)?;

        channel.lock().unwrap().push_back(ScannerStatus::Info("scanning...".to_string()));
        debug!("Sent 'scanning' to channel");

        let step = self.bandwidth / 2;
        let start = self.from as usize - self.bandwidth; //(self.from * 1e6) as usize - self.bandwidth;
        let end = self.to as usize + self.bandwidth*2; //(self.to * 1e6) as usize + self.bandwidth * 2;

        // TODO: align to 512
        let sample_count = (self.dwell_ms * self.samplerate) / 1000;
        let buffer_size = calculate_aligned_buffer_size(sample_count);
        debug!("Buffer size {} bytes, {} samples", buffer_size, sample_count);

        let fftPlan = Plan::new(sample_count as usize);

        {
            driver.set_sample_rate(self.samplerate as u32).unwrap();
            driver.set_tuner_bandwidth(self.bandwidth as u32).unwrap();
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
        // Maybe self.bandidth could be a basic unit of measure?
        //
        // TODO: research delay needed to avoid empty buffer at the start after change frequency
        //

        debug!("Scanning from {} to {}", self.from, self.to);
        let mut freq: usize = start;
        let mut i = 0;

        //print!("Estimated lines: {} {}\n", (end - start) as f64/ step as f64, ((end - start) as f64 / step as f64).ceil());

        while freq <= end {
            let buffer: Vec<u8>;
            {
                //let driver = s.device.as_mut().unwrap();
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
                debug!("> {}", freq as f64/1e6);
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

            let psd = dsp::psd(&complex_dft);

            let _fft_step = 1.0 / (self.dwell_ms as f32 / 1000.0);
            // TODO: send data
            /*
            let mut samples = samples.lock().unwrap();
            for c in psd.into_iter() {
                samples.samples.push(c);
            }
            */
            channel.lock().unwrap().push_back(ScannerStatus::Data(psd));
        }

        {
            let mut channel = channel.lock().unwrap();
            channel.push_back(ScannerStatus::Info("Scanning complete".to_string()));
            channel.push_back(ScannerStatus::Complete);
        }
        Ok(())
    }

    fn refresh(&self) {
        /*let rescaled  = {
            let samples = self.samples.lock().unwrap();
            if samples.samples.len() ==0 {
                return;
            }
            rescale(self.width, self.height, &samples.samples);
        };
        let data_qv = rescaled.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        self.plot(data_qv.into());
        */
    }
}

fn calculate_aligned_buffer_size(samples: usize) -> usize {
    // a sample is a complex byte, thus 2 bytes per sample
    let bytes = samples * 2;
    // TODO: align to ^2 because FFTW works the fastest than
    return bytes + bytes % 512;
}
