use crate::plugin_chain::PluginChain;
use log::*;
use pyo3::prelude::*;
use rack::prelude::*;
use rack::vst3::Vst3Plugin;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};
use biquad::*;

pub mod plugin_chain;
pub mod traits;
// pub mod do_daw_test;

pub const N_CHANNELS: usize = 4;
pub const N_EFFECTS: usize = 3;
pub const SAMPLE_RATE: usize = 48000;
pub const BUFFER_FRAMES: usize = 128;

pub type SinglePlugin = Vst3Plugin;
pub type Sample = f32;

#[pyclass(unsendable)]
pub struct Mixer {
    // _thread_jh: JoinHandle<()>,
    // send: Sender<()>,
    // recv: Receiver<Vec<u8>>,
    _device: OutputDevice,
    pub channels: Arc<Vec<Arc<RwLock<PluginChain>>>>,
    /// global effects on the output of all channels. these get applied after the channels are
    /// mixed together.
    pub effects: Arc<RwLock<Vec<SinglePlugin>>>,
    // TODO: add sequence to mixer
}

#[pymethods]
impl Mixer {
    #[new]
    pub fn new() -> Self {
        env_logger::builder().format_timestamp(None).init();
        let channels: Arc<Vec<_>> = Arc::new(
            (0..N_CHANNELS).map(|_| Arc::new(RwLock::new(PluginChain::default()))).collect()
        );
        let effects:Arc<RwLock<Vec<SinglePlugin>>> = Arc::new(RwLock::new(Vec::new()));

        // start audio output and store join_handle wrapper in self

        let params = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: SAMPLE_RATE,
            channel_sample_count: BUFFER_FRAMES,
        };

        // start audio playback
        let _device = run_output_device(params, {
            let channels = channels.clone();
            let effects = effects.clone();

            // Cutoff and sampling frequencies
            let f0 = ((20_000 + 20) / 2).hz();
            let fs = SAMPLE_RATE.hz();
            let coeffs =
                Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
            let mut allpass = DirectForm1::<f32>::new(coeffs);

            move |data| {
                // Create audio buffers (planar format - separate buffer per channel)
                let mut pre_master_buss: Vec<Vec<Sample>> =
                    (0..BUFFER_FRAMES).map(|_| Vec::with_capacity(N_CHANNELS)).collect();

                channels
                    .iter()
                    .for_each(|locked_channel| {
                        if let Err(e) = locked_channel
                            .write()
                            .map(|mut unlocked_channel| unlocked_channel.get_samples(BUFFER_FRAMES).map(|samples| samples.into_iter().enumerate().for_each(|(i, sample)| pre_master_buss[i].push(sample)))) {
                            error!("{e}");
                        }
                    });

                // if let Err(e) = channels[0]
                //     .write()
                //     .map(|mut unlocked_channel| unlocked_channel.get_samples(BUFFER_FRAMES).map(|samples| samples.into_iter().enumerate().for_each(|(i, sample)| pre_master_buss[i].push(sample)))) {
                //     error!("{e}");
                // }


                let pre_master_bus: Vec<Sample> = pre_master_buss.iter().map(|samples| { 
                    let sample: Sample =  samples.into_iter().sum();
                    allpass.run(sample).tanh()
                    // sample
                }).collect();


                let post_master_bus = if let Ok(mut effects) = effects.write() {
                    let mut input = pre_master_bus.clone();

                    effects.iter_mut().for_each(|effect| {
                        let mut output = vec![0.0f32; BUFFER_FRAMES];

                        if let Err(e) = effect.process(&[&input], &mut [&mut output], BUFFER_FRAMES) {
                            warn!(
                                "effect plugin @ path {} attempted to produce output but failed with error {e}",
                                effect.info().path.display()
                            );
                        } else {
                            input = output.clone();
                        }
                    });
                    
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

        Self { channels, effects, _device }
    }

    /// returns a list of available effects
    pub fn get_plugin_list(&self) -> Vec<(String, PathBuf)> {
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
    pub fn set_note(&mut self, section: usize, channel: usize, step: usize) {
        // TODO: add note to sequence
    }

    /// sets the instrument plugin for channel, to synth. the synth param is a pathbuf gotten from
    /// Mixer.get_plugin_list
    pub fn set_instrument(&mut self, channel_i: usize, synth: String) {
        // let do_set_instrument = |channel: | {
        //             };

        let Some(Ok(channel)) = &mut self
            .channels
            .get(channel_i)
            .map(|lock_writer| lock_writer.write())
        else {
            return;
        };

        if let Some(plugin) = load_plugin(&synth) {
            info!(
                "setting the instrument for channel no. {channel_i} to the plugin, {synth}, from path, {}", plugin.info().path.display()
            );
            channel.sound_gen = Some(plugin);
        }
    }

    /// adds an effect to an effect chain if channel is None the effect is on the mixer not a
    /// channel
    pub fn add_effect(&mut self, channel: Option<usize>, location: usize, effect: String) {
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

            if let Some(plugin) = load_plugin(&effect) {
                if location < channel.effects.len() {
                    channel.effects.insert(location, plugin);
                } else {
                    channel.effects.push(plugin);
                }
            }
        } else if let Ok(mut effects) = self.effects.write()
            && effects.len() < N_EFFECTS
        {
            if let Some(plugin) = load_plugin(&effect) {
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
    pub fn rm_effect(&mut self, channel: Option<usize>, effect: usize) {
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

// fn load_plugin(plugin_path: PathBuf) -> Option<SinglePlugin> {
//     // Create scanner and scan for plugins
//     let scanner = Scanner::new().ok()?;
//     let plugins = scanner.scan().ok()?;
//     let synth_info = plugins.iter().find(|p| p.path == plugin_path)?;
//     let plugin = scanner.load(&synth_info);
//
//     if plugin.is_err() {
//         warn!(
//             "failed to find the plugihttps://doc.rust-lang.org/beta/core/iter/fn.chain.htmln @ path {}.",
//             plugin_path.display()
//         )
//     }
//
//     plugin.ok()
// }

pub fn load_plugin(plugin_name: &str) -> Option<SinglePlugin> {
    // Create scanner and scan for plugins
    let scanner = Scanner::new().ok()?;
    let plugins = scanner.scan().ok()?;
    let synth_info = plugins.iter().find(|p| p.name == plugin_name)?;
    let plugin = scanner.load(&synth_info).map(|mut plugin| 
        {
            if let Err(e) = plugin.initialize(SAMPLE_RATE as f64, BUFFER_FRAMES) {
                warn!("plugin failed to init. {e}");
            } else {
                info!("loaded and inited plugin: {plugin_name}");
            }

            plugin
        });  // .map(|plugin| plugin.initialize(SAMPLE_RATE as f64, BUFFER_FRAMES));

    if plugin.is_err() {
        warn!(
            "failed to find the plugin with name {plugin_name}."
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

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use rack::*;

    use crate::{Mixer, N_CHANNELS};

    #[test]
    fn audio_ouptut() {
        // env_logger::builder().format_timestamp(None).init();

        let mut mixer = Mixer::new();
        let chan = 0;
        
        for chan in 0..N_CHANNELS {
            mixer.set_instrument(chan, "Wt Synth".into());
        }

        let on_events = vec![
            MidiEvent::note_on(60, 100, 0, 0), // Middle C (C4), velocity 100, channel 0
            MidiEvent::note_on(64, 100, 0, 0), // E4
            MidiEvent::note_on(67, 100, 0, 0), // G4
        ];
        // info!("instrument loaded");


        if let Ok(mut channel) = mixer.channels[chan].write() {
            if let Some(sound_gen) = &mut channel.sound_gen {
                if let Err(e) = sound_gen.send_midi(&on_events) {
                    panic!("sending midi failed with error {e}");
                }
            } else {
                panic!("no sound generator");
            }
        } else {
            panic!("failed to write channel {chan}");
        }

        sleep(Duration::from_secs(5));

        // if let Some(plugin) = &mixer.channels[chan].read().unwrap().sound_gen { 
        //     log::debug!("sound_gen = {:?}", plugin.info().name.clone());
        // }

        // panic!("foobar");
    }
}
