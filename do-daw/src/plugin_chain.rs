use crate::{Sample, SinglePlugin};
use log::*;
use pyo3::prelude::*;
use rack::PluginInstance;

#[pyclass]
#[derive(Default)]
pub struct PluginChain {
    pub sound_gen: Option<SinglePlugin>,
    pub effects: Vec<SinglePlugin>,
}

// impl crate::traits::GenSamples for PluginChain {
impl PluginChain {
    pub fn get_samples(&mut self, buffer_size: usize) -> Option<Vec<Sample>> {
        if let Some(sound_gen) = &mut self.sound_gen {
            // trace!(
            //     "sound generator is located @ {}",
            //     sound_gen.info().path.display()
            // );

            let mut output = vec![0.0f32; buffer_size];
            if let Err(e) = sound_gen.process(&[], &mut [&mut output], buffer_size) {
                warn!(
                    "plugin @ path {} attempted to produce output but failed with error {e}",
                    sound_gen.info().path.display()
                );
            }

            let mut input = output.clone();

            self.effects.iter_mut().for_each(|effect| {
                let mut output = vec![0.0f32; buffer_size];

                if let Err(e) = effect.process(&[&input], &mut [&mut output], buffer_size) {
                    warn!(
                        "effect plugin @ path {} attempted to produce output but failed with error {e}",
                        effect.info().path.display()
                    );
                } else {
                    input = output.clone();
                }
            });

            Some(input)
        } else {
            None
        }
    }
}
