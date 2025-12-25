use crate::plugin_chain::PluginChain;
use log::*;
use pyo3::prelude::*;
use rack::prelude::*;
use rack::vst3::Vst3Plugin;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tinyaudio::{OutputDeviceParameters, run_output_device};
use biquad::*;

pub mod plugin_chain;
pub mod traits;

const N_CHANNELS: usize = 4;
const N_EFFECTS: usize = 3;
const SAMPLE_RATE: usize = 48000;
const BUFFER_FRAMES: usize = 128;

pub type SinglePlugin = Vst3Plugin;
pub type Sample = f32;

#[pyclass]
pub struct Mixer {
    // _thread_jh: JoinHandle<()>,
    // send: Sender<()>,
    // recv: Receiver<Vec<u8>>,
    channels: Arc<[Arc<RwLock<PluginChain>>; N_CHANNELS]>,
    /// global effects on the output of all channels. these get applied after the channels are
    /// mixed together.
    effects: Arc<RwLock<Vec<SinglePlugin>>>,
    // TODO: add sequence to mixer
}

#[pymethods]
impl Mixer {
    #[new]
    fn new() -> Self {
        env_logger::builder().format_timestamp(None).init();
        let channels = Arc::new(
            array_macro::array![Arc::new(RwLock::new(PluginChain::default())); N_CHANNELS],
        );
        let effects:Arc<RwLock<Vec<SinglePlugin>>> = Arc::new(RwLock::new(Vec::new()));

        // TODO: start audio output and store join_handle wrapper in self

        let params = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: SAMPLE_RATE,
            channel_sample_count: BUFFER_FRAMES,
        };

        // start audio playback
        let _device = run_output_device(params, {
            let channels = channels.clone();
            let effects = effects.clone();

            

            move |data| {
                // Create audio buffers (planar format - separate buffer per channel)
                // let mut output_audio_buffer = vec![0.0f32; BUFFER_FRAMES];

                // if let Ok(mut plugin) = plugin.write() {
                //     if let Err(e) =
                //         plugin.process(&[], &mut [&mut output_audio_buffer], buffer_frames)
                //     {
                //         println!("processing audio failed with error: {e}");
                //     }
                // )
            
                // Cutoff and sampling frequencies
                let f0 = ((20_000 + 20) / 2).hz();
                let fs = SAMPLE_RATE.hz();
                let coeffs =
                    Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
                let mut allpass = DirectForm1::<f32>::new(coeffs);

                let channels_outputs: Vec<Vec<Sample>> = channels
                    .par_iter()
                    .filter_map(|locked_channel| {
                        if let Some(thing) = locked_channel
                            .write()
                            .map(|mut unlocked_channel| unlocked_channel.get_samples(BUFFER_FRAMES))
                            .ok()
                        {
                            thing
                        } else {
                            None
                        }
                    })
                    .collect();

                // let samples = channels_outputs.step_

                let mut pre_master_buss =
                    array_macro::array![Vec::<Sample>::with_capacity(channels_outputs.len()); BUFFER_FRAMES];

                for (chan_i, output_buf) in channels_outputs.into_iter().enumerate() {
                    for (sample_i, sample) in output_buf.into_iter().enumerate() {
                        pre_master_buss[sample_i][chan_i] = sample;
                    }
                }

                let pre_master_bus: Vec<Sample> = pre_master_buss.iter().map(|samples| { 
                    let sample: Sample =  samples.into_iter().sum();
                    allpass.run(sample * 0.75) * 0.5
                }).collect();


            let post_master_bus = if let Ok(mut effects) = effects.write() {
                let mut input = pre_master_bus.clone();

                for effect in effects.iter_mut() {
                    let mut output = vec![0.0f32; BUFFER_FRAMES];

                    if let Err(e) = effect.process(&[&input], &mut [&mut output], BUFFER_FRAMES) {
                        warn!(
                            "effect plugin @ path {} attempted to produce output but failed with error {e}",
                            effect.info().path.display()
                        );
                    } else {
                        input = output.clone();
                    }
                }
                
                input
            } else {
                pre_master_bus
            };




            for (samples, value) in data
                    .chunks_mut(params.channels_count)
                    .zip(post_master_bus)
                {
                    for sample in samples {
                        *sample = value;
                    }
                }
            }
        })
        .expect("failed to start audio thread...");

        Self { channels, effects }
    }

    /// returns a list of available effects
    fn get_plugin_list(&self) -> Vec<(String, PathBuf)> {
        // Create scanner and scan for plugins
        let Ok(scanner) = Scanner::new() else {
            return Vec::new();
        };
        let Ok(plugins) = scanner.scan() else {
            return Vec::new();
        };

        plugins.into_iter().map(|p| (p.name, p.path)).collect()
    }

    /// sets the note at step of channel in section
    fn set_note(&mut self, section: usize, channel: usize, step: usize) {
        // TODO: add note to sequence
    }

    /// sets the instrument plugin for channel, to synth. the synth param is a pathbuf gotten from
    /// Mixer.get_plugin_list
    fn set_instrument(&mut self, channel_i: usize, synth: PathBuf) {
        // let do_set_instrument = |channel: | {
        //             };

        let Some(Ok(mut channel)) = self
            .channels
            .get(channel_i)
            .map(|lock_writer| lock_writer.write())
        else {
            return;
        };

        if let Some(plugin) = load_plugin(synth.clone()) {
            info!(
                "setting the instrument for channel no. {channel_i} to the plugin from path {}",
                synth.display()
            );
            channel.sound_gen = Some(plugin);
        }
    }

    /// adds an effect to an effect chain if channel is None the effect is on the mixer not a
    /// channel
    fn add_effect(&mut self, channel: Option<usize>, location: usize, effect: PathBuf) {
        if let Some(channel_i) = channel {
            let Some(Ok(mut channel)) = self
                .channels
                .get(channel_i)
                .map(|lock_writer| lock_writer.write())
            else {
                return;
            };

            if channel.effects.len() >= N_EFFECTS {
                return;
            }

            if let Some(plugin) = load_plugin(effect) {
                if location < channel.effects.len() {
                    channel.effects.insert(location, plugin);
                } else {
                    channel.effects.push(plugin);
                }
            }
        } else if let Ok(mut effects) = self.effects.write()
            && effects.len() < N_EFFECTS
        {
            if let Some(plugin) = load_plugin(effect) {
                if location < effects.len() {
                    effects.insert(location, plugin);
                } else {
                    effects.push(plugin);
                }
            }
        }
    }

    /// removes an effect to an effect chain if channel is None the effect is on the mixer not
    /// a channel
    fn rm_effect(&mut self, channel: Option<usize>, effect: usize) {
        if let Some(channel_i) = channel {
            if let Some(Ok(mut channel)) = self
                .channels
                .get(channel_i)
                .map(|lock_writer| lock_writer.write())
            {
                channel.effects.remove(effect);
            }
        } else if let Ok(mut effects) = self.effects.write() {
            effects.remove(effect);
        }
    }
}

fn load_plugin(plugin_path: PathBuf) -> Option<SinglePlugin> {
    // Create scanner and scan for plugins
    let scanner = Scanner::new().ok()?;
    let plugins = scanner.scan().ok()?;
    let synth_info = plugins.iter().find(|p| p.path == plugin_path)?;
    let plugin = scanner.load(&synth_info);

    if plugin.is_err() {
        warn!(
            "failed to find the plugin @ path {}.",
            plugin_path.display()
        )
    }

    plugin.ok()
}

// #[pyfunction]
// fn run() -> Mixer {
//     env_logger::builder().format_timestamp(None).init();
//     let channels = [const { None }; N_CHANNELS];
//     let effects = Vec::new();
//
//     Mixer { channels, effects }
// }

/// A Python module implemented in Rust.
#[pymodule]
fn do_daw(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mixer>()?;

    // m.add_function(wrap_pyfunction!(run, m)?)?;

    Ok(())
}
