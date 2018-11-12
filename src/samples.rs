#[derive(Debug)]
pub(crate) struct Samples {
    pub samples: Vec<f64>,
    pub range_left: usize,
    pub range_right: usize,
    bandwidth: usize,
    f_sampling: usize,
    dwell_ms:  usize,
}

impl Samples {
    pub fn new(f_sampling: usize, range_left: usize, range_right: usize, dwell_ms: usize, bandwidth: usize) -> Samples {
        let range = range_right - range_left;
        let data_points = range * f_sampling * dwell_ms / 1000 / bandwidth;
        Samples {samples: Vec::with_capacity(data_points),
            range_left: range_left,
            range_right: range_right,
            bandwidth: bandwidth,
            f_sampling: f_sampling,
            dwell_ms: dwell_ms
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_append() {
        //let s = Spectrum::new(10);
    }

    #[test]
    fn can_left_join() {

    }

    #[test]
    fn can_merge() {

    }

    #[test]
    fn can_left_interlap() {

    }

    #[test]
    fn can_right_interlap() {

    }
}
