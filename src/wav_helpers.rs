use std::sync::atomic::{Ordering};

pub fn calculate_rms() {
    if let Some(generated_rms) = find_rms_value(crate::GENERATE_PATH) {
        if let Some(recorded_rms) = find_rms_value(crate::RECORD_PATH) {
            let ratio = recorded_rms/generated_rms;
            let gain = 20.0 * ratio.log10();
            crate::RMS_GAIN.store(f64::to_bits(gain), Ordering::SeqCst);
        }
    }
}

fn find_rms_value(filename: &str) -> Option<f64> {
    let mut reader = hound::WavReader::open(filename).unwrap();
    let sqr_sum = match reader.spec().sample_format {
        hound::SampleFormat::Int => reader.samples::<i16>().fold(0.0, |sqr_sum, s| {
                let sample = s.unwrap() as f64;
                sqr_sum + sample * sample
            }),
        hound::SampleFormat::Float => reader.samples::<f32>().fold(0.0, |sqr_sum, s| {
                let sample = s.unwrap() as f64;
                sqr_sum + sample * sample
            }),
    };
    let rms_value = (sqr_sum / reader.len() as f64).sqrt();
    Some(rms_value)
}
