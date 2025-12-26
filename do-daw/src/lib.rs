use crate::{mixer::Mixer, step_sequencer::StepSequencer};
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
pub const BUFFER_FRAMES: usize = 128;

pub type SinglePlugin = Vst3Plugin;
pub type Sample = f32;

#[pyfunction]
/// Builds the Mixer, Step-Sequencer and makes threads for them where applicable
fn run() -> (StepSequencer, Mixer) {
    env_logger::builder().format_timestamp(None).init();
    let (mixer, dev) = Mixer::new();

    (StepSequencer::new(mixer.clone(), dev), mixer)
}

/// A Python module implemented in Rust.
#[pymodule]
fn do_daw(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mixer>()?;
    m.add_class::<StepSequencer>()?;

    m.add_function(wrap_pyfunction!(run, m)?)?;

    Ok(())
}
