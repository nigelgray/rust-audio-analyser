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

fn main() -> Result<(), failure::Error> {
    audio_helpers::record_audio();
    wav_helpers::calculate_rms();
    fft_helpers::calculate_peak_frequency();

    println!("Gain is {:.2} dB", f64::from_bits(RMS_GAIN.load(Ordering::Relaxed)));
    println!("Generated THD+N {:.4} %", f64::from_bits(GENERATED_THD.load(Ordering::Relaxed)));
    println!("Generated Peak is {:.0} Hz", f32::from_bits(GENERATED_PEAK_FREQUENCY.load(Ordering::Relaxed)));
    println!("Recorded THD+N {:.4} %", f64::from_bits(RECORDED_THD.load(Ordering::Relaxed)));
    println!("Recorded Peak is {:.0} Hz", f32::from_bits(RECORDED_PEAK_FREQUENCY.load(Ordering::Relaxed)));
    Ok(())
}
