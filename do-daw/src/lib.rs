use std::path::PathBuf;

use pyo3::prelude::*;
use rack::prelude::*;
use rack::vst3::Vst3Plugin;

const N_CHANNELS: usize = 4;
const N_EFFECTS: usize = 3;

pub type SinglePlugin = Vst3Plugin;

#[pyclass]
pub struct PluginChain {
    pub sound_gen: SinglePlugin,
    pub effects: Vec<SinglePlugin>,
}

#[pyclass]
pub struct Mixer {
    // _thread_jh: JoinHandle<()>,
    // send: Sender<()>,
    // recv: Receiver<Vec<u8>>,
    channels: [Option<PluginChain>; N_CHANNELS],
    /// global effects on the output of all channels. these get applied after the channels are
    /// mixed together.
    effects: Vec<SinglePlugin>,
    // TODO: add sequence to mixer
}

#[pymethods]
impl Mixer {
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
    fn set_instrument(&mut self, channel: usize, synth: PathBuf) {
        let Some(Some(channel)) = self.channels.get_mut(channel) else {
            return;
        };

        if let Some(plugin) = load_plugin(synth) {
            channel.sound_gen = plugin;
        }
    }

    /// adds an effect to an effect chain if channel is None the effect is on the mixer not a
    /// channel
    fn add_effect(&mut self, channel: Option<usize>, location: usize, effect: PathBuf) {
        if let Some(channel_i) = channel {
            let Some(Some(channel)) = self.channels.get_mut(channel_i) else {
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
        } else if self.effects.len() < N_EFFECTS {
            if let Some(plugin) = load_plugin(effect) {
                if location < self.effects.len() {
                    self.effects.insert(location, plugin);
                } else {
                    self.effects.push(plugin);
                }
            }
        }
    }

    /// removes an effect to an effect chain if channel is None the effect is on the mixer not
    /// a channel
    fn rm_effect(&mut self, channel: Option<usize>, effect: usize) {
        if let Some(channel_i) = channel {
            if let Some(Some(channel)) = self.channels.get_mut(channel_i) {
                channel.effects.remove(effect);
            }
        } else {
            self.effects.remove(effect);
        }
    }
}

fn load_plugin(plugin_path: PathBuf) -> Option<SinglePlugin> {
    // Create scanner and scan for plugins
    let scanner = Scanner::new().ok()?;
    let plugins = scanner.scan().ok()?;
    let synth_info = plugins.iter().find(|p| p.path == plugin_path)?;

    scanner.load(&synth_info).ok()
}

#[pyfunction]
fn run() -> Mixer {
    env_logger::builder().format_timestamp(None).init();
    let channels = [const { None }; N_CHANNELS];
    let effects = Vec::new();

    Mixer { channels, effects }
}

/// A Python module implemented in Rust.
#[pymodule]
fn do_daw(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mixer>()?;

    m.add_function(wrap_pyfunction!(run, m)?)?;

    Ok(())
}
