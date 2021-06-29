use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod audio_helpers;
mod wav_helpers;
mod fft_helpers;

use std::sync::atomic::{AtomicUsize, AtomicU32, AtomicU64, Ordering};

const GENERATE_PATH: &'static str = "generated.wav";
const RECORD_PATH: &'static str = "recorded.wav";

static FREQUENCY: AtomicUsize = AtomicUsize::new(1000);
static RMS_GAIN: AtomicU64 = AtomicU64::new(0);
static GENERATED_THD: AtomicU64 = AtomicU64::new(0);
static GENERATED_PEAK_FREQUENCY: AtomicU32 = AtomicU32::new(0);
static RECORDED_THD: AtomicU64 = AtomicU64::new(0);
static RECORDED_PEAK_FREQUENCY: AtomicU32 = AtomicU32::new(0);

#[pyfunction]
fn set_frequency(freq: usize) {
    FREQUENCY.store(freq, Ordering::SeqCst);
}

#[pyfunction]
fn process_audio() {
    audio_helpers::record_audio();
    wav_helpers::calculate_rms();
    fft_helpers::calculate_peak_frequency();
}

#[pyfunction]
fn get_rms_gain() -> f64 {
    f64::from_bits(RMS_GAIN.load(Ordering::Relaxed))
}

#[pyfunction]
fn get_generated_thd() -> f64 {
    f64::from_bits(GENERATED_THD.load(Ordering::Relaxed))
}

#[pyfunction]
fn get_generated_peak_frequency() -> f32 {
    f32::from_bits(GENERATED_PEAK_FREQUENCY.load(Ordering::Relaxed))
}

#[pyfunction]
fn get_recorded_thd() -> f64 {
    f64::from_bits(RECORDED_THD.load(Ordering::Relaxed))
}

#[pyfunction]
fn get_recorded_peak_frequency() -> f32 {
    f32::from_bits(RECORDED_PEAK_FREQUENCY.load(Ordering::Relaxed))
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn rust_audio_tester(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(set_frequency))?;
    m.add_wrapped(wrap_pyfunction!(process_audio))?;
    m.add_wrapped(wrap_pyfunction!(get_rms_gain))?;
    m.add_wrapped(wrap_pyfunction!(get_generated_thd))?;
    m.add_wrapped(wrap_pyfunction!(get_generated_peak_frequency))?;
    m.add_wrapped(wrap_pyfunction!(get_recorded_thd))?;
    m.add_wrapped(wrap_pyfunction!(get_recorded_peak_frequency))?;

    Ok(())
}
