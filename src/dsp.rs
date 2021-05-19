use std::f64::consts::PI;

/// Normalize and center 8-bits signal to [-1.0; +1.0] range.
/// Convert to complex from (real,imaginary) input format.
pub fn rtl_import(rtl_buffer: &Vec<u8>, buff_len: usize, complex: &mut [f64]) {
    // rtl data is (real,imaginary), 0-255 range
    let mut i = 0;
    let mut im_sum = 0_f64;
    let mut re_sum = 0_f64;
    while i < buff_len {
        let re = ((rtl_buffer[i] as i16) - 127) as f64 / 127_f64;
        let im = ((rtl_buffer[i+1] as i16) - 127) as f64 / 127_f64;
        complex[i] = re;
        complex[i+1] = im;

        // TODO: when Rust is better with iterators, return iterator and chain DC correction into external map() call
        re_sum += re;
        im_sum += im;

        i += 2;
    }

    re_sum /= (buff_len / 2) as f64;
    im_sum /= (buff_len / 2) as f64;

    // apply DC correction
    i = 0;
    while i < buff_len {
        complex[i] -= re_sum;
        complex[i+1] -= im_sum;
        i+=2;
    }
}

/// Normalize and center 8-bits signal to [-1; +1] range.
pub fn normalize(rtl_buffer: &Vec<u8>, normalized: &mut [f64]) {
    assert_eq!(rtl_buffer.len(), normalized.len());
    for i in 0..rtl_buffer.len() {
        normalized[i] = ((rtl_buffer[i] as i16) - 127) as f64 / 127_f64;
    }
}

pub fn dc_correction(data: &mut Vec<f64>) {
    let mut re_sum = 0_f64;
    let mut im_sum = 0_f64;
    for i in 0..data.len() / 2 {
        re_sum += data[i*2];
        im_sum += data[i*2 + 1];
    }

    re_sum /= (data.len() / 2) as f64;
    im_sum /= (data.len() / 2) as f64;

    for i in 0..data.len()/2 {
        data[i*2] -= re_sum;
        data[i*2+1] -= im_sum;
    }
}

pub fn rtl_to_abs(rtl_buffer: &Vec<u8>, len: usize) -> Vec<f64> {
    let mut i = 0;
    let mut res = Vec::with_capacity(len/2);
    while i < len {
        let re = (rtl_buffer[i] as i16) - 127;
        let im = (rtl_buffer[i+1] as i16) - 127;
        let abs = ((re*re + im*im) as f64).sqrt();
        res.push(abs);
        i += 2;
    }
    return res;
}

pub fn complex_to_abs(complex: &[f64]) -> Vec<f64> {
    let len = complex.len();
    let mut res = Vec::with_capacity(len/2);
    let mut i = 0;
    while i < len {
        let re = complex[i];
        let im = complex[i+1];
        let abs = (re*re + im*im).sqrt();
        res.push(abs);
        i += 2;
    }
    return res;
}

/// https://www.mathworks.com/help/signal/ug/power-spectral-density-estimates-using-fft.html
pub fn psd(dft: &[f64]) -> Vec<f64> {
    let k = 1.0 / (2.0 * PI * (dft.len()/2) as f64);
    let mut ret = Vec::with_capacity(dft.len()/2);
    // TODO: preserve (f64,f64) for complex numbers Don't forget to correct len() everywhere.
    for i in 0..dft.len()/2 {
        let re = dft[i*2];
        let im = dft[i*2+1];
        let square_norm = re*re + im*im;
        let psd_log = 10.0 * (square_norm * k).log10();
        ret.push(psd_log);
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::File;
    use std::str::FromStr;
    use crate::fftw::*;
    use crate::iterators::*;
    use std::io::{BufReader, BufWriter};
    use std::fmt::Display;
    use futures::{StreamExt, FutureExt};

    const epsilon: f64 = 0.000000000000001;
    // Tolerate bigger error in fft
    const fft_epsilon: f64 = 0.00000000001;
                              //.58514513639

    fn read_matlab_complex(fname: &str) -> Vec<Vec<(f64,f64)>> {
        let mut f = BufReader::new(File::open(fname).expect(&format!("Failed to open file: {}", fname)));
        f.lines().
            map(|l| l.unwrap()).
            filter(|l| !l.starts_with('#') && !l.is_empty()).
            //nth(0).unwrap().
            map(|l|{
                l.split(") (").
                map(|n| n.trim_matches(|x| x == ' ' || x == '(' || x == ')')).
                map(|nn| {
                    let mut parts = nn.split(',');
                    let re = f64::from_str(parts.next().unwrap()).unwrap();
                    let im = f64::from_str(parts.next().unwrap()).unwrap();
                    (re, im)
                }).collect::<Vec<_>>()
            }).collect()
    }

    fn read_matlab<T: std::str::FromStr>(file: &str) -> Vec<Vec<T>>
        where T: std::str::FromStr,
            <T as std::str::FromStr>::Err: std::fmt::Debug
    {
        let mut f = BufReader::new(File::open(file).expect("Can't open file"));
        f.lines().
            map(|l| l.unwrap()).
            filter(|line| !line.starts_with('#') && line.len() > 0).
            map(|l| {
                l.split(' ').
                    filter(|s| !s.is_empty()).map(|s| {
                    s.parse().unwrap()
                }).collect()
            }).collect()
    }

    fn compare_matrix<T: Display>(expected: &[Vec<T>], given: &[Vec<T>], eq: fn(&T,&T) -> bool) {
        if expected.len() != given.len() {
            panic!("Row count is different. Expected {} but got {}", expected.len(), given.len());
        }
        for i in 0..expected.len() {
            if expected[i].len() != given[i].len() {
                panic!("Column count different at row{}: expected {} but got {}", i, expected[i].len(), given[i].len());
            }
            for j in 0..expected[i].len() {
                if !eq(&expected[i][j], &given[i][j]) {
                    panic!("Value different at row {} column {}. Expected {} but got {}", i, j, expected[i][j], given[i][j])
                }
            }
        }
    }

    fn flatten_complex(vec: &Vec<(f64,f64)>) -> Vec<f64> {
        let mut flat = vec![];
        for (re,im) in vec {
            flat.push(*re);
            flat.push(*im);
        }
        flat
    }

    fn write_matlab_from_complex_flat(file: &str, matrix: &Vec<Vec<f64>>) {
        let mut file = BufWriter::new(File::create(file).unwrap());
        file.write_all(b"# Created by rtl-scanner\n");
        file.write_all(b"# name: fft_test\n");
        file.write_all(b"# type: complex matrix\n");
        writeln!(file, "# rows: {}", matrix.len());
        writeln!(file, "# columns: {}", matrix[0].len()/2 );

        for row in matrix {
            for i in 0..row.len()/2 {
                let re = row[i*2];
                let im = row[i*2+1];
                write!(file, " ({},{})", re, im);
            }
            writeln!(file);
        }
    }

    #[test]
    fn compare_to_matlab() {
        let n = read_matlab::<f64>("data/n.mat");
        let dump = read_matlab::<u8>("data/dump.mat");

        let mut normalized: Vec<Vec<f64>> = dump.iter().map(|row| {
            let mut normalized = vec![0_f64; 64_000];
            normalize(&row, &mut normalized);
            normalized
        }).collect();

        compare_matrix(&n, &normalized, |a,b| {(a-b).abs() < epsilon});

        for row in &mut normalized {
            dc_correction(row);
        }

        let ca = read_matlab_complex("data/ca.mat");
        let ca: Vec<_> = ca.iter().map(|r| flatten_complex(r)).collect();
        compare_matrix(&ca, &normalized, |a,b| (a-b).abs() < epsilon);

        let mut fft: Vec<Vec<f64>> = vec![];
        let fft_plan = Plan::new(ca[0].len()/2 as usize);
        let input = fft_plan.get_input();
        let output: &[f64] = fft_plan.get_output();

        for row in ca {
            input.copy_from_slice(&row);
            fft_plan.execute();
            fft.push(output.to_owned());
        }

        let fa = read_matlab_complex("data/fa.mat");
        let fa: Vec<_> = fa.iter().map(|r| flatten_complex(r)).collect();

        write_matlab_from_complex_flat("data/fft_test.mat",&fft);

        compare_matrix(&fa, &fft, |a,b| (a-b).abs() < fft_epsilon);

        // "p" is single column vector
        let p: Vec<f64> = read_matlab("data/p.mat").iter().
            map(|r| r[0]).collect();
    }
}