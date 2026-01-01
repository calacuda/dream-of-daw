use crate::{N_CHANNELS, N_SECTIONS, step_sequencer::N_STEPS};
use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum UiSector {
    #[default]
    Steps,
    Sections,
    ChannelSelect,
    Controls,
    // Settings,
    // Playback,
    // Scale,
    // BPM,
    /// scale, BPM, Settings, play, & pause.
    BottomRight,
    // Playback,
}

#[pyclass]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Cursor {
    #[pyo3(get, set)]
    pub sector: UiSector,
    #[pyo3(get)]
    pub index: isize,
}

#[pymethods]
impl Cursor {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: find a way to "step" this state machine by button presses

    // pub fn enter_controls(&mut self) {
    //     self.index = 0;
    //     self.sector = UiSector::Controls;
    // }

    pub fn up(&mut self) {
        match self.sector {
            UiSector::Steps => {
                if self.index < (N_STEPS / 2) as isize {
                    self.index += (N_STEPS / 2) as isize;
                } else {
                    self.index -= (N_STEPS / 2) as isize;
                }
            }
            UiSector::Sections => self.index = (self.index - 1) % (N_SECTIONS as isize),
            UiSector::ChannelSelect => self.index = (self.index - 1) % (N_CHANNELS as isize),
            UiSector::Controls => {
                if self.index < 4 {
                    self.index += 4;
                } else {
                    self.index -= 4;
                }
            }
            UiSector::BottomRight => {
                if self.index < 3 {
                    self.index = (self.index - 1) % 5
                } else {
                    self.index = 2;
                }
            }
        }
    }

    pub fn down(&mut self) {
        match self.sector {
            UiSector::Steps => {
                if self.index < (N_STEPS / 2) as isize {
                    self.index += (N_STEPS / 2) as isize;
                } else {
                    self.index -= (N_STEPS / 2) as isize;
                }

                self.index %= N_STEPS as isize;
            }
            UiSector::Sections => self.index = (self.index + 1) % (N_SECTIONS as isize),
            UiSector::ChannelSelect => self.index = (self.index + 1) % (N_CHANNELS as isize),
            UiSector::Controls => {
                if self.index < 4 {
                    self.index += 4;
                } else {
                    self.index -= 4;
                }
            }
            UiSector::BottomRight => {
                if self.index < 3 {
                    self.index = (self.index + 1) % 5
                } else {
                    self.index = 0;
                }
            }
        }
    }

    pub fn left(&mut self) {
        match self.sector {
            UiSector::Steps => {
                let jump_line = self.index >= (N_STEPS / 2) as isize;
                self.index -= if self.index == 0 {
                    ((N_STEPS / 2) as isize - 1) * -1
                } else {
                    1
                };
                self.index %= (N_STEPS / 2) as isize;

                if jump_line {
                    self.index += (N_STEPS / 2) as isize;
                }
            }
            UiSector::Sections => {}
            UiSector::ChannelSelect => {
                self.index = 0;
                self.sector = UiSector::Sections;
            }
            UiSector::Controls => {
                if self.index < 4 {
                    self.index -= 1;
                    self.index %= 4;
                } else {
                    self.index -= 1;
                    self.index %= 4;
                    self.index += 4;
                }
            }
            UiSector::BottomRight => {
                if self.index == 4 {
                    self.index -= 1;
                }
            }
        }
    }

    pub fn right(&mut self) {
        match self.sector {
            UiSector::Steps => {
                let jump_line = self.index >= (N_STEPS / 2) as isize;
                self.index += 1;
                self.index %= (N_STEPS / 2) as isize;

                if jump_line {
                    self.index += (N_STEPS / 2) as isize;
                }
            }
            UiSector::Sections => {
                self.index = 0;
                self.sector = UiSector::ChannelSelect;
            }
            UiSector::ChannelSelect => {
                self.index = 0;
                self.sector = UiSector::Steps;
            }
            UiSector::Controls => {
                if self.index < 4 {
                    self.index += 1;
                    self.index %= 4;
                } else {
                    self.index += 1;
                    self.index %= 4;
                    self.index += 4;
                }
            }
            UiSector::BottomRight => {
                if self.index == 4 {
                    self.index += 1;
                }
            }
        }
    }
}
