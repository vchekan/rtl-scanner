use std::f64::consts::PI;
use num::complex::*;

/// https://www.mathworks.com/help/signal/ug/power-spectral-density-estimates-using-fft.html
pub fn psd(dft: &Vec<Complex64>) -> Vec<f64> {
    let k = 1.0 / (2.0 * PI * dft.len() as f64);
    dft.iter().
        map(|x| x.norm()).
        map(|x| x*x*k).
        map(|x| 10.0*x.log10()).
        // TODO: how to return an iterator?
        collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::complex::*;
    use std::io::prelude::*;
    use std::fs::File;
    use std::str::FromStr;
    use fftw::*;
    use iterators::*;

    fn read_matlab_complex(fname: &str) -> Vec<Complex64> {
        let mut f = File::open(fname).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s.lines().
            filter(|l| !l.starts_with('#')).
            nth(0).unwrap().
            split(") (").
            map(|n| n.trim_matches(|x| x == ' ' || x == '(' || x == ')')).
            map(|nn| {
                let mut parts = nn.split(',');
                let re = f64::from_str(parts.next().unwrap()).unwrap();
                let im = f64::from_str(parts.next().unwrap()).unwrap();
                Complex64::new(re, im)
            }).collect::<Vec<_>>()
    }

    #[test]
    fn compare_to_matlab() {
        let samples = &read_matlab_complex("data/signal.mat");
        println!("samples: {}", samples.len());
        let fft = Plan::new(samples.len());
        println!("fft: {:?}", fft);
        let input = fft.get_input();
        let output = fft.get_output();
        for i in 0..samples.len() { input[i*2] = samples[i].re; input[i*2+1] = samples[i].im }
        fft.execute();
        println!("samples: {:?}", &samples[0..10]);
        println!("input: {:?}", &input[0..10]);
        println!("output: {:?}", &output[0..10]);

        let complex_dft = output.iter().cloned().tuples().
            map(|(re, im)| Complex64::new(re, im)).
            collect::<Vec<_>>();
        let scan = psd(&complex_dft);
        println!("scan: {:?}", scan);
    }
}