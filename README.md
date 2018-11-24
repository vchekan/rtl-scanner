RTL radio spectrum scanner
Toy project to learn Rust

Notes:
Integer FFT: http://www.jjj.de/fft/fftpage.html

Psd:
https://www.mathworks.com/help/signal/ug/power-spectral-density-estimates-using-fft.html

Goos quality analyzer:
http://www.kerrywong.com/2014/11/16/testing-an-rtl-sdr-spectrum-analyzer/

DC removal:
http://www.embedded.com/design/configurable-systems/4007653/DSP-Tricks-DC-Removal

TODO:
    Chart:
        Elasic size
        Grid
        Axis labels
    * Switch to SoapySDR api
    * SIMD
        Compare performance/respurces to fftw
    * Events and rendering are polled in tight loop. Need to rework to use `events_loop.run_forever()`
        and inject charts update statistics.
    * FFTW is in f64 by default, it would be nice to do f32 arithmetics instead or even in integrer domain