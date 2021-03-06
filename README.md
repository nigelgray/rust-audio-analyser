![](https://images.pond5.com/audio-signal-animation-different-white-footage-082028002_iconl.jpeg)

# rust-audio-analyser
Simple Cross-platform Audio Analyser

## Aims

Measuring the audio performance of audio equipment usually involves expensive test equipment.

With "Pro-sumer" audio cards becoming both cheap and relatively high-quality, it should be possible to use them as cheap audio analysers.

Rust provides a good cross-platform audio library (cpal) and good FFT libraries (such as rustfft), making it a favourable option.

## Equipment

This codebase was developed using a Focusrite Scarlett 2i2 USB soundcard, but should work on any modern soundcard.

Frequency Response = 20Hz - 20kHz ± 0.1dB

Dynamic Range = 110.5dB (A-weighted)

THD+N = <0.002%

![](https://mixdownmag.com.au/wp-content/uploads/2016/08/focusrite_0.jpg)

## Tests

There are a number of common tests that you would want to run to validate the audio performance.

https://en.wikipedia.org/wiki/Audio_system_measurements

### THD+N (Total Harmonic Distortion + Noise)

How much noise the audio equipment adds.

https://en.wikipedia.org/wiki/Total_harmonic_distortion

### Cross-talk

How much of the left channel bleeds into the right (and vice-versa).

https://en.wikipedia.org/wiki/Crosstalk

### Frequency Response

How flat the frequency response looks across the test range.

https://en.wikipedia.org/wiki/Frequency_response

## Sample Output

Using a loop-back test with some instrument cables, the measurements look promising.

```
PS C:\workspace\rust-audio-analyser> cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.12s
     Running `target\debug\rust-audio-analyser.exe`
Input device: Focusrite USB (Focusrite USB Audio)
Input format: Format { channels: 2, sample_rate: SampleRate(48000), data_type: F32 }
Output device: Focusrite USB (Focusrite USB Audio)
Output format: Format { channels: 2, sample_rate: SampleRate(48000), data_type: F32 }
Gain is 0.93 dB
Generated THD+N 0.0000 %
Generated Peak is 1000 Hz
Recorded THD+N 0.1032 %
Recorded Peak is 1000 Hz
```

