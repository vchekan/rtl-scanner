use libc::*;
use num::complex::Complex;

//pub type fftw_plan = *mut u8;
//pub type fftw_complex = *mut u8;
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
    buffer: Vec<u8>,    // input and output buffer, fftw's in-place transform mode
}

impl Plan {
    pub fn new(n: i32) {
        let c_buff = unsafe { fftw_malloc(n as size_t)} ;    // TODO: check for null result
        let buffer = unsafe { Vec::from_raw_parts(c_buff, 0 , (n*4) as usize)};
        let plan_ptr = unsafe {fftw_plan_dft_1d(n, c_buff, c_buff, FFTW_FORWARD, FFTW_ESTIMATE)};

    }
}

impl Drop for Plan {
    fn drop(&mut self) {
        if self.fftw_plan != null {
            fftw_destroy_plan(self.fftw_plan);
            self.fftw_plan = 0;
        }

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