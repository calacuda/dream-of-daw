use crate::{N_CHANNELS, mixer::Mixer};
use pyo3::prelude::*;
use std::{
    sync::atomic::{AtomicBool, AtomicUsize},
    thread::JoinHandle,
};
use tinyaudio::OutputDevice;

const N_STEPS: usize = 16;

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct StepState {
    note: Option<u8>,
    velocity: u8,
    /// per-channel
    channel: u8,
    /// position of the mod wheel
    mod_whl: f32,
    /// position of the pitch wheel
    pitch_bend: f32,
    macto_1: Option<(u8, f32)>,
    macto_2: Option<(u8, f32)>,
    macto_3: Option<(u8, f32)>,
    macto_4: Option<(u8, f32)>,
}

impl Default for StepState {
    fn default() -> Self {
        Self {
            note: None,
            velocity: 64,
            channel: 0,
            mod_whl: 0.0,
            pitch_bend: 0.0,
            macto_1: None,
            macto_2: None,
            macto_3: None,
            macto_4: None,
        }
    }
}

#[pyclass]
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct StepSequence {
    pub steps: [StepState; N_STEPS],
}

#[pymethods]
impl StepSequence {
    pub fn __getitem__(&mut self, i: usize) -> StepState {
        self.steps[i % N_STEPS]
    }

    pub fn __setitem__(&mut self, i: usize, state: StepState) {
        self.steps[i % N_STEPS] = state
    }
}

#[pyclass(unsendable)]
pub struct StepSequencer {
    // pub mixer: PyCell<Mixer>,
    // #[pyo3(get)]
    pub mixer: Mixer,
    _device: OutputDevice,
    pub steps: Vec<StepSequence>,
    pub step_i: AtomicUsize,
    pub playing: AtomicBool,
    // _jh: JoinHandle<()>,
}

// #[pymethods]
impl StepSequencer {
    // #[new]
    pub fn new(mixer: Mixer, _device: OutputDevice) -> Self {
        // let (mixer, _device) = Mixer::new();
        let step_i = 0.into();
        let playing = false.into();
        let steps = vec![StepSequence::default(); N_CHANNELS];

        Self {
            mixer,
            _device,
            steps,
            step_i,
            playing,
        }
    }

    /// sets the note at step of channel in section
    pub fn set_note(&mut self, section: usize, channel: usize, step: usize) {
        // TODO: add note to sequence
    }
}

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use rack::*;

    use crate::{N_CHANNELS, mixer::Mixer, step_sequencer::StepSequencer};

    #[test]
    fn audio_ouptut() {
        // env_logger::builder().format_timestamp(None).init();

        let (mixer, dev) = Mixer::new();
        let mut seq = StepSequencer::new(mixer, dev);
        let chan = 0;

        for chan in 0..N_CHANNELS {
            seq.mixer.set_instrument(chan, "Wt Synth".into());
        }

        let on_events = vec![
            MidiEvent::note_on(60, 100, 0, 0), // Middle C (C4), velocity 100, channel 0
            MidiEvent::note_on(64, 100, 0, 0), // E4
            MidiEvent::note_on(67, 100, 0, 0), // G4
        ];
        // info!("instrument loaded");

        if let Ok(mut channel) = seq.mixer.channels[chan].write() {
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

        if let Some(plugin) = &seq.mixer.channels[chan].read().unwrap().sound_gen {
            log::debug!("sound_gen = {:?}", plugin.info().name.clone());
        }

        // panic!("foobar");
    }
}
