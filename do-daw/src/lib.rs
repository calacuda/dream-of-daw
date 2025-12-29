use crate::{
    mixer::Mixer,
    step_sequencer::{StepSequence, StepSequencer, StepState},
};
use pyo3::prelude::*;
use rack::vst3::Vst3Plugin;

pub mod plugin_chain;
pub mod traits;
// pub mod do_daw_test;
pub mod mixer;
pub mod step_sequencer;

pub const N_CHANNELS: usize = 4;
pub const N_EFFECTS: usize = 3;
pub const N_SECTIONS: usize = 8;
pub const SAMPLE_RATE: usize = 48000;
pub const BUFFER_FRAMES: usize = 512;

pub type SinglePlugin = Vst3Plugin;
pub type Sample = f32;

/// Builds the Mixer, Step-Sequencer and makes threads for them where applicable
#[pyfunction]
fn run() -> (StepSequencer, Mixer) {
    env_logger::builder().format_timestamp(None).init();
    let (mixer, dev) = Mixer::new();

    (StepSequencer::new(mixer.clone(), dev), mixer)
}

#[pyfunction]
fn midi_note(midi_note: usize) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;
    let octave: isize = octave as isize - 1;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

/// A Python module implemented in Rust.
#[pymodule]
fn do_daw(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mixer>()?;
    m.add_class::<StepSequencer>()?;
    m.add_class::<StepSequence>()?;
    m.add_class::<StepState>()?;

    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(midi_note, m)?)?;

    Ok(())
}
