#RTL radio spectrum scanner
Toy project to learn Rust

###Notes:
Integer FFT: http://www.jjj.de/fft/fftpage.html. Implement with SIMD?

Psd:
https://www.mathworks.com/help/signal/ug/power-spectral-density-estimates-using-fft.html

Good quality analyzer:
http://www.kerrywong.com/2014/11/16/testing-an-rtl-sdr-spectrum-analyzer/

DC removal:
http://www.embedded.com/design/configurable-systems/4007653/DSP-Tricks-DC-Removal

###TODO:
[ ] SelectedDeviceLens implementation is a hack, rethink it  
* Split crates into CLI, GUI, sys and lib
* Chart
  * Elasic size
  * Grid
  * Axis labels
* Process Icon
* Implement Stop button
* Switch to SoapySDR api
* Validate PSD results against matlab
* SIMD
    Compare performance/respurces to fftw
* Events and rendering are polled in tight loop. Need to rework to use `events_loop.run_forever()`
    and inject charts update statistics.
* FFTW is in f64 by default, it would be nice to do f32 arithmetics instead or even in integer domain
* dynamically re-scan devices
* detect frequency ranges
* detect direct sampling
* https://crates.io/crates/rustfft/
* Web rendering: https://makepad.dev/
