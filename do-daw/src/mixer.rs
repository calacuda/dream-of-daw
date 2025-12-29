use crate::plugin_chain::PluginChain;
use crate::{Sample, SinglePlugin, BUFFER_FRAMES, N_CHANNELS, N_EFFECTS, SAMPLE_RATE};
use log::*;
use pyo3::prelude::*;
use rack::prelude::*;
use rayon::{current_num_threads, prelude::*};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};
use biquad::*;

struct Multizip<T>(Vec<T>);

impl<T> Iterator for Multizip<T>
where
    T: Iterator,
{
    type Item = Vec<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let item: Vec<Option<T::Item>> = self.0.iter_mut().map(|iter| iter.next()).collect();
        let item: Option<Vec<T::Item>> = item.into_iter().collect();

        item
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Mixer {
    // _thread_jh: JoinHandle<()>,
    // send: Sender<()>,
    // recv: Receiver<Vec<u8>>,
    // _device: OutputDevice,
    pub channels: Arc<Vec<Arc<RwLock<PluginChain>>>>,
    /// global effects on the output of all channels. these get applied after the channels are
    /// mixed together.
    pub effects: Arc<RwLock<Vec<SinglePlugin>>>,
}

impl Mixer {
    // #[new]
    pub fn new() -> (Self, OutputDevice) {
        let channels: Arc<Vec<_>> = Arc::new(
            (0..N_CHANNELS).map(|_| Arc::new(RwLock::new(PluginChain::default()))).collect()
        );
        let effects:Arc<RwLock<Vec<SinglePlugin>>> = Arc::new(RwLock::new(Vec::new()));

        // start audio output
        let params = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: SAMPLE_RATE,
            channel_sample_count: BUFFER_FRAMES,
        };

        // start audio playback
        let device = run_output_device(params, {
            let channels = channels.clone();
            let effects = effects.clone();

            // Cutoff and sampling frequencies
            let f0 = ((20_000 + 20) / 2).hz();
            let fs = SAMPLE_RATE.hz();
            let coeffs =
                Coefficients::<f32>::from_params(Type::AllPass, fs, f0, Q_BUTTERWORTH_F32).unwrap();
            let mut allpass = DirectForm1::<f32>::new(coeffs);
            // let mut allpass = AllPass::new(1.0, SAMPLE_RATE, 0.5);
            // let chunk_size = BUFFER_FRAMES / current_num_threads();
            info!("BUFFER_FRAMES = {BUFFER_FRAMES}");
            // info!("num parallel = {:?}", std::thread::available_parallelism());
            info!("num threads (default) = {}", current_num_threads());
            // info!("chunk_size = {chunk_size}");
            // rayon::ThreadPoolBuilder::new().use_current_thread().num_threads(3).build_global().unwrap();
            // rayon::ThreadPoolBuilder::new().use_current_thread().build_global().unwrap();
            // info!("num_threads (customized) = {}", current_num_threads());

            move |data| {
                // Create audio buffers
                // let mut pre_master_buss: Vec<Vec<Sample>> =
                //     (0..BUFFER_FRAMES).map(|_| Vec::with_capacity(N_CHANNELS)).collect();

                // let mut generated_samples = channels
                //     .par_iter()
                //     .filter_map(|locked_channel| {
                //         let chan_samples = locked_channel
                //             .write()
                //             .map(|mut unlocked_channel| unlocked_channel.get_samples(BUFFER_FRAMES));
                //
                //         match chan_samples {
                //             Ok(samples) => samples,
                //             Err(e) => {
                //                 error!("{e}");
                //                 None
                //             }
                //         }
                //     });
                //
                // generated_samples.for_each(|chan_samples| {
                //     chan_samples.into_iter().enumerate().for_each(|(i, sample)| pre_master_buss[i].push(sample));
                // });
               
                // let mut m_zip = {
                let pre_master_bus: Vec<Sample> = {
                    // debug!("new buffer");

                    let  m_zip = {
                        let samples_by_channel: Vec<std::vec::IntoIter<Sample>> = channels
                            // .iter()
                            // .filter(|locked_channel| {
                            //     locked_channel
                            //         .read()
                            //         .is_ok_and(|unlocked_channel| unlocked_channel.sound_gen.is_some()) 
                            // }).into_iter()
                            // .collect::<Vec<_>>()
                            .par_iter()
                            .filter_map(|locked_channel| {
                                let chan_samples = locked_channel
                                    .write()
                                    .map(|mut unlocked_channel| unlocked_channel.get_samples(BUFFER_FRAMES));

                                match chan_samples {
                                    Ok(samples) => samples.map(|samples| samples.into_iter()),
                                    Err(e) => {
                                        error!("{e}");
                                        None
                                    }
                                }
                            }).collect();
                        Multizip(samples_by_channel)
                    };

                    // debug!("made buffer");
                    // let chunked_samples: Vec<_> = std::iter::from_fn(move || {
                    //     Some(m_zip.by_ref().take(chunk_size).collect()).filter(|chunk: &Vec<_>| !chunk.is_empty())
                    // }).collect();

                    m_zip.into_iter().map(|samples: Vec<Sample>| { 
                        let sample: Sample = samples.iter().sum();
                        // allpass.tick(sample.into()).tanh() as f32
                        allpass.run(sample).tanh()
                        // sample.tanh()
                    }).collect()
                    
                    // chunked_samples.par_iter().map(|chunk: &Vec<Vec<Sample>>| { 
                    //     chunk.into_iter().map(move |samples| {
                    //         // let sample: Sample =  samples.into_iter().sum();
                    //         let sample: Sample = samples.into_iter().sum();
                    //         // allpass.run(sample).tanh()
                    //         sample.tanh()
                    //     }).collect::<Vec<_>>()
                    // }).flatten().collect()
                };

                // debug!("normalized samples");

                // let chunked_samples: Vec<_> = std::iter::from_fn(move || {
                //     Some(m_zip.by_ref().take(chunk_size).collect()).filter(|chunk: &Vec<_>| !chunk.is_empty())
                // }).collect();
                // let pre_master_bus: Vec<Sample> = chunked_samples.par_iter().map(|chunk| { 
                // let pre_master_bus: Vec<Sample> = m_zip.map(|samples| { 
                //     // chunk.into_iter().map(move |samples| {
                //         // let sample: Sample =  samples.into_iter().sum();
                //         let sample: Sample = samples.into_iter().sum();
                //         // allpass.run(sample).tanh()
                //         sample.tanh()
                //     // }).collect::<Vec<_>>()
                // }).collect();

                // debug!("made pre_master_bus");

                // let (rms, peak) = analyze_buffer(&pre_master_bus);
                // debug!("pre-effects => RMS={:6.4} Peak={:6.4}", rms, peak);

                let post_master_bus = if let Ok(mut effects) = effects.write() {
                    // let mut input = pre_master_bus.clone();
                    // let mut input = pre_master_bus;

                    // effects.iter_mut().for_each(|effect| {
                    //     let mut output = vec![0.0f32; BUFFER_FRAMES];
                    //
                    //     if let Err(e) = effect.process(&[&input], &mut [&mut output], BUFFER_FRAMES) {
                    //         warn!(
                    //             "effect plugin @ path {} attempted to produce output but failed with error {e}",
                    //             effect.info().path.display()
                    //         );
                    //     } else {
                    //         input = output;
                    //     }
                    // });
                    //
                    // input

                    let input = pre_master_bus;

                    let output = effects.iter_mut().fold(input, |input, effect| {
                        let mut output = vec![0.0f32; BUFFER_FRAMES];

                        if let Err(e) = effect.process(&[&input], &mut [&mut output], BUFFER_FRAMES) {
                            warn!(
                                "effect plugin @ path {} attempted to produce output but failed with error {e}",
                                effect.info().path.display()
                            );
                        }

                        output
                    });

                    output
                } else {
                    pre_master_bus
                };

                // let (rms, peak) = analyze_buffer(&post_master_bus);
                // debug!("post-effects => RMS={:6.4} Peak={:6.4}", rms, peak);

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

        (Self { channels, effects, /* _device */ }, device)
    }
}

#[pymethods]
impl Mixer {
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

    pub fn play_notes(&mut self, notes: Vec<u8>, channel: usize) {
        let on_events: Vec<MidiEvent> = notes.into_iter().map(|note| MidiEvent::note_on(note, 100, 0, 0)).collect();

        if let Ok(mut channel) = self.channels[channel].write() {
            if let Some(sound_gen) = &mut channel.sound_gen {
                if let Err(e) = sound_gen.send_midi(&on_events) {
                    error!("sending midi failed with error {e}");
                }
            } else {
                error!("no sound generator");
            }
        } else {
            error!("failed to write channel {channel}");
        }
    }

    pub fn stop_notes(&mut self, notes: Vec<u8>, channel: usize) {
        let on_events: Vec<MidiEvent> = notes.into_iter().map(|note| MidiEvent::note_off(note, 100, 0, 0)).collect();

        if let Ok(mut channel) = self.channels[channel].write() {
            if let Some(sound_gen) = &mut channel.sound_gen {
                if let Err(e) = sound_gen.send_midi(&on_events) {
                    error!("sending midi failed with error {e}");
                }
            } else {
                error!("no sound generator");
            }
        } else {
            error!("failed to write channel {channel}");
        }
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

#[allow(dead_code)]
/// Calculate RMS and peak levels for left and right channels (planar format)
fn analyze_buffer(audio: &[f32]) -> (f32, f32) {
    let frames = audio.len();

    let mut sum_right = 0.0f32;
    let mut peak_right = 0.0f32;

    for i in 0..frames {
        sum_right += audio[i] * audio[i];

        peak_right = peak_right.max(audio[i].abs());
    }

    let rms_right = (sum_right / frames as f32).sqrt();

    (rms_right, peak_right)
}

// #[cfg(test)]
// mod test {
//     use std::{thread::sleep, time::Duration};
//
//     use rack::*;
//
//     use crate::{mixer::Mixer, N_CHANNELS};
//
//     #[test]
//     fn audio_ouptut() {
//         // env_logger::builder().format_timestamp(None).init();
//
//         let (mut mixer, _dev) = Mixer::new();
//         let chan = 0;
//
//         for chan in 0..N_CHANNELS {
//             mixer.set_instrument(chan, "Wt Synth".into());
//         }
//
//         let on_events = vec![
//             MidiEvent::note_on(60, 100, 0, 0), // Middle C (C4), velocity 100, channel 0
//             MidiEvent::note_on(64, 100, 0, 0), // E4
//             MidiEvent::note_on(67, 100, 0, 0), // G4
//         ];
//         // info!("instrument loaded");
//
//
//         if let Ok(mut channel) = mixer.channels[chan].write() {
//             if let Some(sound_gen) = &mut channel.sound_gen {
//                 if let Err(e) = sound_gen.send_midi(&on_events) {
//                     panic!("sending midi failed with error {e}");
//                 }
//             } else {
//                 panic!("no sound generator");
//             }
//         } else {
//             panic!("failed to write channel {chan}");
//         }
//
//         sleep(Duration::from_secs(5));
//
//         if let Some(plugin) = &mixer.channels[chan].read().unwrap().sound_gen { 
//             log::debug!("sound_gen = {:?}", plugin.info().name.clone());
//         }
//
//         // panic!("foobar");
//     }
// }
