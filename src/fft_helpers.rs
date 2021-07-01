use rustfft::FFTplanner;
use num::complex::Complex;

use plotlib::page::Page;
use plotlib::scatter::Scatter;
use plotlib::view::ContinuousView;

use csv::Writer;

use std::sync::atomic::{Ordering};

// TODO: Should be able to get the sampling rate directly from the soundcard format info
const SAMPLE_RATE: usize = 48000;
// Cut some of the first and last samples to ensure the audio is clean
const SAMPLE_SIZE: usize = (crate::SECONDS_TO_RECORD - 1.5 as usize) * SAMPLE_RATE;
const SAMPLE_OFFSET: usize = 0.5 as usize * SAMPLE_RATE;

// This will analyse both the generated and recorded audio
// - Read the audio samples in
// - Trim them down to a window of samples between two zero-cross points
// - Run the FFT calculation
// - Find the fundamental frequency, then use that to calculate the THD+N from the remaining signal
pub fn calculate_peak_frequency() {
    let (mut gen_signal, gen_wave_spec) = read_wav_file(crate::GENERATE_PATH);
    gen_signal = find_zero_crosses(gen_signal);
    if let Some((generated_peak, generated_thd)) = find_spectral_peak(gen_signal, gen_wave_spec, "generated") {
        crate::GENERATED_PEAK_FREQUENCY.store(f32::to_bits(generated_peak), Ordering::SeqCst);
        crate::GENERATED_THD.store(f64::to_bits(generated_thd), Ordering::SeqCst);
    }

    let (mut rec_signal, rec_wave_spec) = read_wav_file(crate::RECORD_PATH);
    rec_signal = find_zero_crosses(rec_signal);
    if let Some((recorded_peak, recorded_thd)) = find_spectral_peak(rec_signal, rec_wave_spec, "recorded") {
        crate::RECORDED_PEAK_FREQUENCY.store(f32::to_bits(recorded_peak), Ordering::SeqCst);
        crate::RECORDED_THD.store(f64::to_bits(recorded_thd), Ordering::SeqCst);
    }
}

// Run an FFT on the audio and detect the maximum frequency
// This will be the fundamental frequency and can be used later for calculating the THD+N (signal vs noise)
fn find_spectral_peak(mut signal: Vec<Complex<f32>>, wave_spec: hound::WavSpec, filename: &str) -> Option<(f32, f64)> {
    let bin = wave_spec.sample_rate as f32 * wave_spec.channels as f32 / signal.len() as f32;

    let frequency = crate::FREQUENCY.load(std::sync::atomic::Ordering::Relaxed);
    // This controls the signal versus noise window we will use for the calculation
    // Currently this is trial-and-error, probably need a more mathmatical way to calcualate it
    let thd_size: usize = 100 + (frequency / 200 as usize);

    let mut spectrum = signal.clone();
    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(signal.len());
    fft.process(&mut signal[..], &mut spectrum[..]);

    save_to_csv(spectrum.clone(), filename, bin);

    let max_peak = spectrum.iter()
        .take(signal.len() / 4)
        .enumerate()
        .max_by_key(|&(_, freq)| freq.norm() as u32);

    let mut signal_strength;
    let mut tone_strength = 0f64;
    let mut thd = 0.0;
    if let Some((i, freq)) = max_peak {
        plot_fft(spectrum.clone(), filename, bin as f64, freq.norm() as f64);

        let half_thd_size = thd_size/2;
        let mut index = if i > half_thd_size { i - half_thd_size } else { 0 };
        for _n in 0..thd_size {
            tone_strength += (spectrum[index].norm() as f64).powi(2);
            index += 1;
        }

        signal_strength = spectrum.iter().take(signal.len()/4).fold(0f64, |sum, s| sum + (s.norm() as f64).powi(2));
        signal_strength = signal_strength.sqrt();
        tone_strength = tone_strength.sqrt();
        thd = 100f64 * (signal_strength - tone_strength)/signal_strength;
    }

    if let Some((i, _)) = max_peak {
        Some((i as f32 * bin, thd))
    } else {
        None
    }
}

fn read_wav_file(filename: &str) -> (Vec<Complex<f32>>, hound::WavSpec) {
    let mut reader = hound::WavReader::open(filename).expect("Failed to open WAV file");
    let wave_spec = reader.spec();

    match wave_spec.sample_format {
        hound::SampleFormat::Int => (reader.samples::<i16>()
                .map(|x| Complex::new(x.unwrap() as f32, 0f32))
                .collect::<Vec<_>>(), wave_spec),
        hound::SampleFormat::Float => (reader.samples::<f32>()
                .map(|x| Complex::new(x.unwrap() as f32, 0f32))
                .collect::<Vec<_>>(), wave_spec),
    }
}

fn plot_fft(spectrum: Vec<Complex<f32>>, filename: &str, bin: f64, max_peak: f64) {

    let log_data: Vec<_> = spectrum.iter()
        .take(spectrum.len() / 4)
        .enumerate()
        .map(|(i,value)| (i as f64 * bin, 20f64 * (value.norm() as f64/max_peak).log10() ))
        .collect();

    let linear_data: Vec<_> = spectrum.iter()
        .take(spectrum.len() / 4)
        .enumerate()
        .map(|(i,value)| (i as f64 * bin, value.norm() as f64))
        .collect();

    let log_slice = Scatter::from_slice(log_data.as_slice());
    let log_view = ContinuousView::new()
        .add(&log_slice)
        .x_label("Frequency")
        .y_label("dB");

    let linear_slice = Scatter::from_slice(linear_data.as_slice());
    let linear_view = ContinuousView::new()
        .add(&linear_slice)
        .x_label("Frequency")
        .y_label("dB");

    // A page with a single view is then saved to an SVG file
    Page::single(&log_view).save(filename.to_owned() + "_log.svg").expect("saving svg");
    Page::single(&linear_view).save(filename.to_owned() + "_linear.svg").expect("saving svg");
}

// Dump the data to a CSV file, so we can load it into a spreadsheet for debugging
fn save_to_csv(spectrum: Vec<Complex<f32>>, filename: &str, bin: f32) {

    let mut wtr = Writer::from_path(filename.to_owned() + ".csv").expect("Couldn't open CSV file");
    for (i,value) in spectrum.iter().take(spectrum.len() / 4).enumerate() {
        wtr.write_record(&[(i as f32 * bin).to_string(), (value.norm() as f32).to_string()]).expect("Couldn't write to CSV");
    }
    wtr.flush().expect("Couldn't flush CSV");
}

// Any FFT calculations need to be done between zero crosses, otherwise the discontinuous data
// will cause havoc with the FFT calc and we'll get a garbage result
fn find_zero_crosses(signal: Vec<Complex<f32>>) -> Vec<Complex<f32>> {
    let mut start_cross = SAMPLE_OFFSET;
    let mut end_cross = SAMPLE_SIZE + SAMPLE_OFFSET;

    let mut positive = signal[start_cross].re >= 0f32;
    while start_cross < signal.len() {
        if (signal[start_cross].re >= 0f32) != positive {
            break;
        }
        start_cross += 1;
    }

    positive = signal[end_cross].re >= 0f32;
    while end_cross < signal.len() {
        if (signal[end_cross].re >= 0f32) != positive {
            break;
        }
        end_cross += 1;
    }

    signal[start_cross..end_cross].to_vec()
}
