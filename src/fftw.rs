use libc::*;
use std::*;

pub static FFTW_FORWARD: c_int = -1;
pub static FFTW_MEASURE: c_uint = 0;
pub static FFTW_ESTIMATE: c_uint = 1 << 6;


#[link(name="fftw3")]
extern {
    pub fn fftw_plan_dft_1d(n: c_int, _in: *mut u8, out: *mut u8, sign: c_int, flags: c_uint) -> *mut u8;
    pub fn fftw_execute(p: *const u8);
    pub fn fftw_destroy_plan(p: *mut u8);
    pub fn fftw_malloc(n: size_t) -> *mut u8;
    pub fn fftw_free(buff: *mut u8);
}

#[derive(Debug)]
pub struct Plan {
    fftw_plan: *mut u8,
    // http://www.fftw.org/fftw3_doc/Complex-One_002dDimensional-DFTs.html#Complex-One_002dDimensional-DFTs
    // The data is an array of type fftw_complex, which is by default a double[2] composed of the
    // real (in[i][0]) and imaginary (in[i][1]) parts of a complex number.
    input: *mut f64,
    output: *mut f64,
    len: usize  // samples count, 2 f64 per sample
}

impl Plan {
    pub fn new(n: usize) -> Plan {
        unsafe {
            // From fftw doc: we recommend using fftw_malloc, which behaves like malloc except that it
            // properly aligns the array when SIMD instructions
            let input = fftw_malloc(n*8*2);
            let output = fftw_malloc(n*8*2);

            if input.is_null() || output.is_null() {panic!("fftw_malloc failed")}
            let plan_ptr = fftw_plan_dft_1d(n as c_int, input, output, FFTW_FORWARD, FFTW_ESTIMATE);
            Plan {
                fftw_plan: plan_ptr,
                input: input as *mut f64,
                output: output as *mut f64,
                len: n
            }
        }
    }

    pub fn execute(&self) {
        unsafe {fftw_execute(self.fftw_plan)}
    }

    pub fn get_input(&self) -> &mut [f64] {
        unsafe {slice::from_raw_parts_mut(self.input, self.len*2) }
    }

    pub fn get_output(&self) -> &[f64] {
        unsafe {slice::from_raw_parts(self.output, self.len*2) }
    }

}

impl Drop for Plan {
    fn drop(&mut self) {
        unsafe {
            fftw_destroy_plan(self.fftw_plan);
            fftw_free(self.input as *mut u8);
            fftw_free(self.output as *mut u8);
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: test against OSMOCOM spectrum analyzer:
    // http://www.kerrywong.com/2014/11/16/testing-an-rtl-sdr-spectrum-analyzer/
    use super::*;
    #[test]
    fn executes() {
        let p = Plan::new(10);
        p.execute();
    }
}