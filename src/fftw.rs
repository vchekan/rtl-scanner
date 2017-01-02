use libc::*;

pub static FFTW_FORWARD: c_int = -1;
pub static FFTW_ESTIMATE: c_uint = 1 << 6;

#[link(name="fftw3")]
extern {
    pub fn fftw_plan_dft_1d(n: c_int, _in: *mut u8, out: *mut u8, sign: c_int, flags: c_uint) -> *mut u8;
    pub fn fftw_destroy_plan(p: *mut u8);
    pub fn fftw_malloc(n: size_t) -> *mut u8;
}

pub struct Plan {
    fftw_plan: *mut u8,
    // http://www.fftw.org/fftw3_doc/Complex-One_002dDimensional-DFTs.html#Complex-One_002dDimensional-DFTs
    // The data is an array of type fftw_complex, which is by default a double[2] composed of the
    // real (in[i][0]) and imaginary (in[i][1]) parts of a complex number.
    input: Vec<f64>,
    output: Vec<f64>,
}

impl Plan {
    pub fn new(n: i32) {
        let c_buff = unsafe { fftw_malloc(n as size_t)} ;    // TODO: check for null result
        let plan_ptr = unsafe {fftw_plan_dft_1d(n, c_buff, c_buff, FFTW_FORWARD, FFTW_ESTIMATE)};

        // TODO:
        // From fftw doc: we recommend using fftw_malloc, which behaves like malloc except that it
        // properly aligns the array when SIMD instructions
        let input: Vec<f64> = Vec::with_capacity((n*2) as usize);
        let output: Vec<f64> = Vec::with_capacity((n*2) as usize);
    }
}

impl Drop for Plan {
    fn drop(&mut self) {
        unsafe { fftw_destroy_plan(self.fftw_plan) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocates() {
        let p = Plan::new(10);
    }
}