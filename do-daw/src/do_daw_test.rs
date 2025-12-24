//! Simple synthesizer example
//!
//! This example demonstrates:
//! - Finding an instrument plugin
//! - Sending MIDI note events
//! - Processing audio to render the notes
//! - Analyzing the output with RMS and peak levels
//!
//! Run with: cargo run --example simple_synth

use rack::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};
use tinyaudio::prelude::*;

pub fn main() -> Result<()> {
    println!("Rack Simple Synthesizer Example");
    println!("================================\n");

    // Create scanner and scan for plugins
    let scanner = Scanner::new()?;
    let plugins = scanner.scan()?;

    println!("Found {} plugins total", plugins.len());

    // Find an instrument plugin (synthesizer)
    let synth_info = plugins
        .iter()
        .find(|p| p.plugin_type == PluginType::Instrument)
        .ok_or_else(|| {
            Error::Other(
            "No instrument plugins found. Install a synthesizer AudioUnit to run this example.\n\
             macOS includes DLSMusicDevice by default. You can also install free synths like:\n\
             - Dexed (DX7 emulator)\n\
             - Surge XT\n\
             - Vital".to_string()
        )
        })?;

    println!("Using instrument: {}", synth_info.name);
    println!("Manufacturer: {}", synth_info.manufacturer);
    println!("Type: {:?}", synth_info.plugin_type);
    println!("Path: {:?}", synth_info.path);
    // println!("Params: {:?}\n", synth_info.);

    // Load the plugin
    let mut plugin = scanner.load(synth_info)?;

    // Initialize with 48kHz sample rate and 512 frame buffer
    let sample_rate = 48000.0;
    let buffer_frames = 128;
    // let buffer_frames = sample_rate as usize;
    plugin.initialize(sample_rate, buffer_frames)?;
    println!("Params Count: {:?}", plugin.parameter_count());
    // let param = 42;
    // let param = 0;
    // for param in 0..42 {
    //     if let Ok(info) = plugin.parameter_info(param) {
    //         println!("Params info: {param} => {:?}", info.name);
    //     }
    // }
    println!("");
    let plugin = Arc::new(RwLock::new(plugin));

    println!("Plugin initialized:");
    println!("  Sample rate: {:.1} Hz", sample_rate);
    println!("  Buffer size: {} frames\n", buffer_frames);

    // Process multiple buffers to let the synth generate audio
    let note_len = 30.0;

    // let mut left_out = vec![0.0f32; buffer_frames];
    // plugin.process(&[], &mut [&mut left_out], buffer_frames)?;
    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: sample_rate as usize,
        channel_sample_count: buffer_frames,
    };

    // start audio playback
    let _device = run_output_device(params, {
        let plugin = plugin.clone();

        move |data| {
            // Create audio buffers (planar format - separate buffer per channel)
            let mut output_audio_buffer = vec![0.0f32; buffer_frames];

            if let Ok(mut plugin) = plugin.write() {
                if let Err(e) = plugin.process(&[], &mut [&mut output_audio_buffer], buffer_frames)
                {
                    println!("processing audio failed with error: {e}");
                }
            }

            for (samples, value) in data
                .chunks_mut(params.channels_count)
                .zip(output_audio_buffer)
            {
                for sample in samples {
                    *sample = value;
                }
            }
        }
    })
    .expect("failed to start audio thread...");

    sleep(Duration::from_secs_f64(0.01));

    // Play a C major chord (C-E-G)
    println!("Playing C major chord (notes 60, 64, 67)...");

    let events = vec![
        MidiEvent::note_on(60, 100, 0, 0), // Middle C (C4), velocity 100, channel 0
        MidiEvent::note_on(64, 100, 0, 0), // E4
        MidiEvent::note_on(67, 100, 0, 0), // G4
    ];

    if let Ok(mut plugin) = plugin.write() {
        plugin.send_midi(&events)?;
        println!("✓ MIDI note on events sent");
    } else {
        println!(
            "✓ MIDI note on events sending error, failed to lock the RwLock with write permissions."
        );
    }

    sleep(Duration::from_secs_f64(note_len));

    // Send note off events to release the notes
    println!("\nReleasing notes...");

    let events = vec![
        MidiEvent::note_off(60, 64, 0, 0),
        MidiEvent::note_off(64, 64, 0, 0),
        MidiEvent::note_off(67, 64, 0, 0),
    ];

    if let Ok(mut plugin) = plugin.write() {
        plugin.send_midi(&events)?;
        println!("✓ MIDI note off events sent");
    } else {
        println!(
            "✓ MIDI note off events sending error, failed to lock the RwLock with write permissions."
        );
    }

    // Process a few more buffers during release phase
    println!("\nProcessing {} buffers during release...", 5);

    sleep(Duration::from_secs_f64(0.3));

    println!("\n✓ Synthesis complete!");
    println!("\nNote: If all RMS/Peak values are 0.0000, the synth may need");
    println!("      different initialization or parameter setup.");

    Ok(())
}

// /// Calculate RMS and peak levels for left and right channels (planar format)
// fn analyze_buffer(left: &[f32], right: &[f32]) -> (f32, f32, f32, f32) {
//     let frames = left.len();
//
//     let mut sum_left = 0.0f32;
//     let mut sum_right = 0.0f32;
//     let mut peak_left = 0.0f32;
//     let mut peak_right = 0.0f32;
//
//     for i in 0..frames {
//         sum_left += left[i] * left[i];
//         sum_right += right[i] * right[i];
//
//         peak_left = peak_left.max(left[i].abs());
//         peak_right = peak_right.max(right[i].abs());
//     }
//
//     let rms_left = (sum_left / frames as f32).sqrt();
//     let rms_right = (sum_right / frames as f32).sqrt();
//
//     (rms_left, rms_right, peak_left, peak_right)
// }
