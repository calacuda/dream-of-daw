//! Simple synthesizer example
//!
//! This example demonstrates:
//! - Finding an instrument plugin
//! - Sending MIDI note events
//! - Processing audio to render the notes
//! - Analyzing the output with RMS and peak levels
//!
//! Run with: cargo run --example simple_synth

use do_daw::do_daw_test as do_daw;
use rack::prelude::*;

fn main() -> Result<()> {
    do_daw::main()
}
