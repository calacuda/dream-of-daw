// use pyo3::prelude::*;
use rack::prelude::*;
use rack::vst3::Vst3Plugin;
use std::sync::{Arc, Mutex, RwLock};

pub mod do_daw_test;

const N_CHANNELS: usize = 4;

// pub type SinglePlugin = Arc<RwLock<Vst3Plugin>>;
pub type SinglePlugin = Vst3Plugin;

// #[pyclass]
pub struct PluginChain {
    pub sound_gen: SinglePlugin,
    pub effects: Vec<SinglePlugin>,
}

// #[pyclass]
pub struct Mixer {
    // _thread_jh: JoinHandle<()>,
    // send: Sender<()>,
    // recv: Receiver<Vec<u8>>,
    channels: [Option<PluginChain>; N_CHANNELS],
}

// #[pymethods]
impl Mixer {
    // fn recv(&self) -> Option<Vec<u8>> {
    //     self.recv.try_iter().last()
    // }
    //
    // fn stop(&self) {
    //     if let Err(e) = self.send.send(()) {
    //         error!("failed to stop bevy {e}")
    //     }
    // }

    fn get_plugin_list(&self) -> Vec<String> {
        Vec::new()
    }
}

// #[pyfunction]
fn run() -> Mixer {
    env_logger::builder().format_timestamp(None).init();
    let channels = [const { None }; N_CHANNELS];

    do_daw_test::main();

    Mixer { channels }
}

// /// A Python module implemented in Rust.
// #[pymodule]
// fn do_daw(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<Mixer>()?;
//
//     m.add_function(wrap_pyfunction!(run, m)?)?;
//     Ok(())
// }
