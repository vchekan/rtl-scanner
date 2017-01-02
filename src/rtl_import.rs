use std::num;

pub fn rtl_import(rtl_buffer: &Vec<u8>, len: usize, output: &mut Vec<f64>) {
    // rtl data is (real,imaginary), 0-255 range

    let mut i = 0;
    while i < len {
        let re = (rtl_buffer[i] as i16) - 127;
        let im = (rtl_buffer[i+1] as i16) - 127;
        output[i] = re as f64;
        output[i+1] = im as f64;
        i += 2;
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