use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum UiSector {
    #[default]
    Steps,
    Sections,
    ChannelSelect,
    Controls,
    Settings,
    Playback,
    Scale,
    BPM,
}

// #[pymethods]
// impl UiSector {
//     fn __eq__(&self, )
// }

#[pyclass]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Cursor {
    #[pyo3(get)]
    pub sector: UiSector,
    #[pyo3(get)]
    pub index: usize,
}

#[pymethods]
impl Cursor {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
    // is_steps

    // TODO: find a way to "step" this state machine by button presses
}
