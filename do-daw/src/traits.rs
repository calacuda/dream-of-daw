use crate::Sample;

pub trait GenSamples {
    // /// create a single mono sample
    // fn get_sample(&mut self) -> Sample;

    /// fill and output buffer
    fn fill_mono_buffer(&mut self, output_buffer: &mut [Sample], buffer_size: usize);
}
