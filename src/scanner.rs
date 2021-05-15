use std::sync::{Arc, Mutex};
use crate::samples;
use crate::fftw::Plan;
use std::thread;
use log::{debug};
use std::fs::File;
use std::io::{Write};
use crossbeam_channel::{Receiver, Sender};
use crate::dsp::rtl_import;

#[derive(Debug)]
pub struct Scanner {
    device_index: i32,
    samples: Arc<Mutex<samples::Samples>>,
    bandwidth: usize,
    dwell_ms: usize,
    samplerate: usize,
    from: u32,
    to: u32,
    dump: Option<File>,
}

pub enum ScannerStatus {
    Info(String),
    Error(String),
    Data(Vec<f64>),
}

impl Scanner {
    pub fn new(device_index: i32, samplerate: usize, from: u32, to: u32, dwell_ms: usize, bandwidth: usize, dump: bool) -> Scanner {
        let device = rtlsdr::open(device_index);
        let dump = if dump {
            Some(File::create("./data/dump.mat").expect("Failed to create dump file"))
        } else {
            None
        };

        Scanner {
            device_index,
            samples: Arc::new(Mutex::new(samples::Samples::new(samplerate, from as usize, to as usize, dwell_ms, bandwidth))),
            bandwidth,
            dwell_ms,
            samplerate,
            from,
            to,
            dump
        }
    }


    // TODO: handle device calls more intelligently than just unwrap(). If device is removed from usb
    // and function call fail, it would cause panic.
    pub fn start(mut self) -> Receiver<ScannerStatus> {
        let (scanner_to_app, app_from_scanner) = crossbeam_channel::bounded(2);
        let worker_handle = thread::spawn(move || {
            if let Err(err) = self.scan(&scanner_to_app) {
                scanner_to_app.send(ScannerStatus::Error(err.to_string()));
            }
        });
        app_from_scanner
    }

    fn scan(&mut self, to_app: &Sender<ScannerStatus>) -> Result<(), rtlsdr::RTLSDRError> {
        let mut driver = rtlsdr::open(self.device_index)?;

        to_app.send(ScannerStatus::Info("scanning...".to_string()));
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

        if let Some(f) = &mut self.dump {
            f.write_all(b"# Created by rtl-scanner\n");
            f.write_all(b"# name: raw_bytes\n");
            f.write_all(b"# type: matrix\n");
            writeln!(f, "# rows: {}", ((end - start) as f64 / step as f64).ceil());
            writeln!(f, "# columns: {}", buffer_size );
        }

        //
        // TODO: think, if it is possible to do frequencies in rational space and not in f64.
        // Maybe self.bandidth could be a basic unit of measure?

        debug!("Scanning from {} to {}", self.from, self.to);
        let mut freq: usize = start;
        let mut i = 0;

        while freq < end {
            let buffer: Vec<u8>;
            {
                driver.set_center_freq(freq as u32).unwrap();
                // TODO: seems like different devices have different delay before start sampling after freq change. Try to autodetect.
                driver.read_sync(4*1024).unwrap();
                // TODO: add borrowed buffer override to rtlsdr driver
                buffer = driver.read_sync(buffer_size as usize).unwrap();
            }

            if let Some(f) = &mut self.dump {
                for b in &buffer {
                    write!(f, "{} ", b);
                }
                writeln!(f);
            }


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
            //let dft_out_ordered = output[output.len()/2..].iter().chain(output[..output.len()/2].iter()).cloned().tuples();
            //let complex_dft = dft_out_ordered.
            //    map(|(re, im)| Complex64::new(re, im)).
            //    // TODO: do not collect but keep propagating Iterator into ::psd
            //    collect::<Vec<_>>();

            //let psd = dsp::psd(&complex_dft);

            let _fft_step = 1.0 / (self.dwell_ms as f32 / 1000.0);
            // TODO: send data
            /*
            let mut samples = samples.lock().unwrap();
            for c in psd.into_iter() {
                samples.samples.push(c);
            }
            */
            //to_app.send(ScannerStatus::Data(psd));
        }

        to_app.send(ScannerStatus::Info("Scanning complete".to_string()));
        Ok(())
    }
}


fn calculate_aligned_buffer_size(samples: usize) -> usize {
    // a sample is a complex byte, thus 2 bytes per sample
    let bytes = samples * 2;
    // TODO: align to ^2 because FFTW works the fastest than
    return bytes + bytes % 512;
}
