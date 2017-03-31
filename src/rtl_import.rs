
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