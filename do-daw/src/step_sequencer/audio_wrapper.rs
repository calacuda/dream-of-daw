use pyo3::prelude::*;
use std::thread::JoinHandle;
use tinyaudio::OutputDevice;

#[pyclass(unsendable)]
pub struct AudioOutputWrapper {
    pub _device: OutputDevice,
    pub _jh: JoinHandle<()>,
}
