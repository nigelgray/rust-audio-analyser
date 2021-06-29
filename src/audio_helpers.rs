use std::sync::atomic::{AtomicBool, Ordering};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
#[cfg(not(feature = "default_card"))]
use cpal::platform::Device;

#[cfg( all(target_os = "linux", not(feature = "default_card")))]
const INPUT_CARD: &'static str = "hw:CARD=Device,DEV=0";

#[cfg(all(not(target_os = "linux"), not(feature = "default_card")))]
const INPUT_CARD: &'static str = "Microphone (USB Advanced Audio Device)";
#[cfg(all(not(target_os = "linux"), not(feature = "default_card")))]
const OUTPUT_CARD: &'static str = "Speakers (USB Advanced Audio Device)";

pub fn record_audio() {
    // Use the default host for working with audio devices.
    let host = cpal::default_host();

    // Setup the default input device and stream with the default input format.
#[cfg(feature = "default_card")]
    let device = host.default_input_device().expect("Failed to get default input device");
#[cfg(not(feature = "default_card"))]
    let device = get_soundcard(INPUT_CARD).expect("Failed to get input device");

    let format = device.default_input_format().expect("Failed to get default input format");
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_input_stream(&device, &format).expect("Input stream error");
    event_loop.play_stream(stream_id).expect("Input Play stream error");

    let spec = wav_spec_from_format(&format);
    let writer = hound::WavWriter::create(crate::RECORD_PATH, spec).expect("Couldn't create file");
    let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

    let gen_writer = hound::WavWriter::create(crate::GENERATE_PATH, spec).expect("Couldn't create file");
    let gen_writer = std::sync::Arc::new(std::sync::Mutex::new(Some(gen_writer)));

    let event_loop_out = host.event_loop();
#[cfg(target_os = "linux")]
    {
        let stream_id_out = event_loop_out.build_output_stream(&device, &format).expect("Output stream error");
        event_loop_out.play_stream(stream_id_out).expect("Output Play stream error");
    }
#[cfg(not(target_os = "linux"))]
    {
#[cfg(feature = "default_card")]
        let device_out = host.default_output_device().expect("Failed to get default output device");
#[cfg(not(feature = "default_card"))]
        let device_out = get_soundcard(OUTPUT_CARD).expect("Failed to get output device");
        println!("Output device: {}", device_out.name().expect("Device name error"));
        let format_out = device_out.default_output_format().expect("Failed to get output format");
        println!("Output format: {:?}", format_out);
        let stream_id_out = event_loop_out.build_output_stream(&device_out, &format_out).expect("Output stream error");
        event_loop_out.play_stream(stream_id_out).expect("Output Play stream error");
    }

    // A flag to indicate that recording is in progress.
    let playing = std::sync::Arc::new(AtomicBool::new(true));
    let playing_2 = playing.clone();
    let gen_writer_2 = gen_writer.clone();
    let sample_rate = format.sample_rate.0 as f32;
    let mut sample_clock = 0f32;

    std::thread::spawn(move || {
        event_loop_out.run(move |id, result| {
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", id, err);
                    return;
                }
            };

            // Produce a sinusoid of maximum amplitude.
            let mut next_value = || {
                sample_clock = (sample_clock + 1.0) % sample_rate;
                let frequency = crate::FREQUENCY.load(Ordering::Relaxed);
                (sample_clock * frequency as f32 * 2.0 * 3.141592 / sample_rate).sin()
            };

            // If we're done playing, return early.
            if !playing_2.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            match data {
                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                    if let Ok(mut guard) = gen_writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for sample in buffer.chunks_mut(format.channels as usize) {
                                let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                                for out in sample.iter_mut() {
                                    *out = value;
                                    writer.write_sample(value as i16).ok();
                                }
                            }
                        }
                    }
                },
                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                    if let Ok(mut guard) = gen_writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for sample in buffer.chunks_mut(format.channels as usize) {
                                let value = (next_value() * std::i16::MAX as f32) as i16;
                                for out in sample.iter_mut() {
                                    *out = value;
                                    writer.write_sample(value).ok();
                                }
                            }
                        }
                    }
                },
                cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    if let Ok(mut guard) = gen_writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for sample in buffer.chunks_mut(format.channels as usize) {
                                let value = next_value();
                                for out in sample.iter_mut() {
                                    *out = value;
                                    writer.write_sample(value).ok();
                                }
                            }
                        }
                    }
                },
                _ => (),
            }
        });
    });

    // A flag to indicate that recording is in progress.
    let recording = std::sync::Arc::new(AtomicBool::new(true));

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    let recording_2 = recording.clone();
    std::thread::spawn(move || {
        event_loop.run(move |id, event| {
            let data = match event {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", id, err);
                    return;
                }
            };

            // If we're done recording, return early.
            if !recording_2.load(Ordering::Relaxed) {
                return;
            }
            // Otherwise write to the wav writer.
            match data {
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::U16(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for sample in buffer.iter() {
                                let sample = cpal::Sample::to_i16(sample);
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::I16(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for &sample in buffer.iter() {
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::F32(buffer) } => {
                    if let Ok(mut guard) = writer_2.try_lock() {
                        if let Some(writer) = guard.as_mut() {
                            for &sample in buffer.iter() {
                                writer.write_sample(sample).ok();
                            }
                        }
                    }
                },
                _ => (),
            }
        });
    });

    // Give the threads time to play/record
    std::thread::sleep(std::time::Duration::from_secs(5));
    recording.store(false, Ordering::Relaxed);
    playing.store(false, Ordering::Relaxed);

    writer.lock().unwrap().take().unwrap().finalize().expect("File write issue");
    gen_writer.lock().unwrap().take().unwrap().finalize().expect("File write issue");
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_format(format: &cpal::Format) -> hound::WavSpec {
    hound::WavSpec {
        channels: format.channels as _,
        sample_rate: format.sample_rate.0 as _,
        bits_per_sample: (format.data_type.sample_size() * 8) as _,
        sample_format: sample_format(format.data_type),
    }
}

#[cfg(not(feature = "default_card"))]
fn get_soundcard(card_name: &str) -> Option<Device> {
    // Use the default host for working with audio devices.
    let host = cpal::default_host();

    let devices = host.devices().expect("");
    println!("Looking for: {}", card_name);
    for (_device_index, device) in devices.enumerate() {
        let device_name = device.name().expect("");
        if device_name == card_name {
            println!("Found: {}", device_name);
            return Some(device);
        }
    }
    return None;
}
