use crate::{N_EFFECTS, Sample, SinglePlugin};
use log::*;
use pyo3::prelude::*;
use rack::PluginInstance;

#[pyclass]
pub struct PluginChain {
    pub sound_gen: Option<SinglePlugin>,
    pub effects: Vec<SinglePlugin>,
    pub volume: f32,
}

impl Default for PluginChain {
    fn default() -> Self {
        Self {
            sound_gen: None,
            effects: Vec::with_capacity(N_EFFECTS),
            volume: 1.0,
        }
    }
}

// impl crate::traits::GenSamples for PluginChain {
impl PluginChain {
    pub fn get_samples(&mut self, buffer_size: usize) -> Option<Vec<Sample>> {
        let sound_gen = self.sound_gen.as_mut()?;
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

        let mut output = self.effects.iter_mut().fold(output, |input, effect| {
            let mut output = vec![0.0f32; buffer_size];

            if let Err(e) = effect.process(&[&input], &mut [&mut output], buffer_size) {
                warn!(
                    "effect plugin @ path {} attempted to produce output but failed with error {e}",
                    effect.info().path.display()
                );
            }

            output
        });

        // attenuate output by self.volume
        output = output.iter().map(|sample| sample * self.volume).collect();

        Some(output)

        // let mut input = output.clone();

        // self.effects.iter_mut().for_each(|effect| {
        //     let mut output = vec![0.0f32; buffer_size];
        //
        //     if let Err(e) = effect.process(&[&input], &mut [&mut output], buffer_size) {
        //         warn!(
        //             "effect plugin @ path {} attempted to produce output but failed with error {e}",
        //             effect.info().path.display()
        //         );
        //     } else {
        //         input = output.clone();
        //     }
        // });

        // Some(input)
    }
}
