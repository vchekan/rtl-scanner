pub fn rescale(width: i32, height: i32, data: &Vec<f64>) -> Vec<f64> {
    let mut res = Vec::with_capacity(width as usize);
    //let max = data.iter().cloned().fold(0./0., f64::max);

    let samples_per_pixel = data.len() as f32 / width as f32;
    let mut max = ::std::f64::MIN;
    let mut min = ::std::f64::MAX;
    for i in 0..width {
        // TODO: averaging shrinks dynamic range. Maybe do percentiles and adjust color intensity accordingly?
        let start_sample = (i as f32 * samples_per_pixel).round() as usize;
        let end_sample = ((i+1) as f32 * samples_per_pixel).round() as usize;
        let avg: f64 = data.as_slice()[start_sample..end_sample].iter().sum::<f64>() / samples_per_pixel as f64;
        res.push(avg);
        if avg > max
            {max = avg;}
        if avg < min && avg != ::std::f64::NEG_INFINITY
            {min = avg;}
    }

    let amplithude = max - min;
    println!("min: {} max: {} amplithude: {}", min, max, amplithude);
    for i in 0..res.len() {res[i] = (res[i] - min) / amplithude * height as f64}

    res
}
