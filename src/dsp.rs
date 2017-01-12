use std::f64::consts::PI;
use num::complex::*;

/// https://www.mathworks.com/help/signal/ug/power-spectral-density-estimates-using-fft.html
pub fn psd(dft: &Vec<Complex64>) -> Vec<f64> {
    let k = 1.0 / (2.0 * PI * dft.len() as f64);
    dft.iter().
        map(|x|  x.norm()).
        map(|x| x*x*k).
        map(|x| 10.0*x.log10()).
        collect()
}