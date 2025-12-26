use crate::mixer::Mixer;
use pyo3::prelude::*;
use tinyaudio::OutputDevice;

#[pyclass(unsendable)]
pub struct StepSequencer {
    // pub mixer: PyCell<Mixer>,
    #[pyo3(set)]
    pub mixer: Mixer,
    _device: OutputDevice,
}

#[pymethods]
impl StepSequencer {
    #[new]
    pub fn new() -> Self {
        let (mixer, _device) = Mixer::new();

        Self { mixer, _device }
    }

    // // fn get_mixer_mut(&mut self) -> PyRefMut<'_, Mixer> {
    // fn get_mixer_mut(&mut self) -> Mixer {
    //     self.mixer.borrow_mut()
    // }

    /// sets the note at step of channel in section
    pub fn set_note(&mut self, section: usize, channel: usize, step: usize) {
        // TODO: add note to sequence
    }
}

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use rack::*;

    use crate::{N_CHANNELS, step_sequencer::StepSequencer};

    #[test]
    fn audio_ouptut() {
        // env_logger::builder().format_timestamp(None).init();

        let mut seq = StepSequencer::new();
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
