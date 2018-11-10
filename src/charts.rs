
pub fn rescale(width: i32, height: i32, data: &Vec<f64>) -> Vec<f64> {
    let mut res = Vec::with_capacity(width as usize);
    //let max = data.iter().cloned().fold(0./0., f64::max);

    let samples_per_pixel = data.len() as f32 / width as f32;
    let mut max = ::std::f64::MIN;
    let mut min = ::std::f64::MAX;
    //let mut max_bucket = ::std::f64::MIN;

    let mut running_max = ::std::f64::MIN;
    let mut end_sample = (1_f32 * samples_per_pixel).round() as usize;
    for i in 0..data.len() {
        // TODO: averaging shrinks dynamic range. Maybe do percentiles and adjust color intensity accordingly?
        //let start_sample = (i as f32 * samples_per_pixel).round() as usize;
        //let end_sample = ((i+1) as f32 * samples_per_pixel).round() as usize;
        //let avg: f64 = data.as_slice()[start_sample..end_sample].iter().sum::<f64>() / (end_sample - start_sample) as f64;

        if running_max < data[i] {
            running_max = data[i];
        }

        if i > end_sample {
            res.push(running_max);
            end_sample = (res.len() as f32 * samples_per_pixel).round() as usize;
            
            running_max = ::std::f64::MIN;
        }


        //res.push(avg);
        if data[i] > max
            {max = data[i];}
        if data[i] < min && data[i] != ::std::f64::NEG_INFINITY
            {min = data[i];}
    }

    let amplitude = max - min;
    for i in 0..res.len() {res[i] = (res[i] - min) / amplitude * height as f64}

    println!("rescaled[{}:{}] {:?}", width, samples_per_pixel, res);

    res
}
