use crate::{
    N_CHANNELS, N_SECTIONS, mixer::Mixer, step_sequencer::audio_wrapper::AudioOutputWrapper,
};
use log::*;
use pyo3::prelude::*;
use rack::prelude::*;
use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{sleep, spawn},
    time::Duration,
};
use tinyaudio::OutputDevice;

pub const N_STEPS: usize = 16;

pub mod audio_wrapper;

#[pyclass(get_all)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct StepState {
    note: Option<u8>,
    velocity: u8,
    /// per-channel
    channel: u8,
    /// position of the mod wheel
    mod_whl: f32,
    /// position of the pitch wheel
    pitch_bend: f32,
    macro_1: Option<(u8, f32)>,
    macro_2: Option<(u8, f32)>,
    macro_3: Option<(u8, f32)>,
    macro_4: Option<(u8, f32)>,
}

impl Default for StepState {
    fn default() -> Self {
        Self {
            note: None,
            velocity: 64,
            channel: 0,
            mod_whl: 0.0,
            pitch_bend: 0.0,
            macro_1: None,
            macro_2: None,
            macro_3: None,
            macro_4: None,
        }
    }
}

#[pyclass]
#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct StepSequence {
    pub steps: [StepState; N_STEPS],
}

#[pymethods]
impl StepSequence {
    pub fn __getitem__(&mut self, i: usize) -> StepState {
        self.steps[i % N_STEPS]
    }

    pub fn __setitem__(&mut self, i: usize, state: StepState) {
        self.steps[i % N_STEPS] = state
    }
}

#[pyclass]
pub struct StepSequencer {
    pub mixer: Mixer,
    pub steps: Arc<[Arc<[RwLock<StepSequence>]>]>,
    pub step_i: Arc<AtomicUsize>,
    pub section_i: Arc<AtomicUsize>,
    pub bpm: Arc<AtomicUsize>,
    pub playing: Arc<AtomicBool>,
}

impl StepSequencer {
    pub fn new(mixer: Mixer, _device: OutputDevice) -> (Self, AudioOutputWrapper) {
        let step_i: Arc<AtomicUsize> = Arc::new((N_STEPS - 1).into());
        let section_i: Arc<AtomicUsize> = Arc::new(0.into());
        let playing: Arc<AtomicBool> = Arc::new(false.into());
        let steps: Vec<Arc<[RwLock<StepSequence>]>> = (0..N_SECTIONS)
            .map(|_| {
                let steps: Vec<RwLock<StepSequence>> = (0..N_CHANNELS)
                    .map(|_| RwLock::new(StepSequence::default()))
                    .collect();
                steps.into()
            })
            .collect();
        let steps: Arc<[Arc<[RwLock<StepSequence>]>]> = steps.into();

        let bpm: Arc<AtomicUsize> = Arc::new(60.into());

        let _jh = spawn({
            let mixer = mixer.clone();
            let steps = steps.clone();
            let step_i = step_i.clone();
            let playing = playing.clone();
            let section_i = section_i.clone();
            let bpm = bpm.clone();

            move || {
                do_run_sequence(mixer, steps, step_i, section_i, playing, bpm);
            }
        });

        (
            Self {
                mixer,
                steps,
                step_i,
                section_i,
                bpm,
                playing,
            },
            AudioOutputWrapper { _device, _jh },
        )
    }
}

#[pymethods]
impl StepSequencer {
    /// sets the note at step of channel in section
    pub fn set_note(&mut self, channel_i: usize, step_i: usize, note: Option<u8>) -> bool {
        let section_i = self.section_i.load(Ordering::Relaxed);
        let Some(Some(Ok(Some(new_note)))) = self.steps.get(section_i).map(|section| {
            section.get(channel_i).map(|channel| {
                channel.write().map(|mut channel| {
                    channel.steps.get_mut(step_i).map(|step| {
                        debug!("setting section {section_i}, channel {channel_i}, step {step_i}, to note {note:?}"); 
                        step.note = note;
                        step.note
                    })
                })
            })
        }) else {
            return false;
        };

        new_note == note
    }

    /// sets the note at step of channel in section
    pub fn edit_note(&mut self, channel_i: usize, step_i: usize, note: i8) {
        let section_i = self.section_i.load(Ordering::Relaxed);

        self.steps.get(section_i).map(|section| {
            section.get(channel_i).map(|channel| {
                channel.write().map(|mut channel| {
                    channel.steps.get_mut(step_i).map(|step| {
                        debug!("editing section {section_i}, channel {channel_i}, step {step_i}, by value {note:?}"); 
                        step.note.as_mut().map(|num| if note.abs() as u8 <= *num && !(note < 0 && *num == 24) {
                            *num = ((*num as i8) + note).abs() as u8 % 108;

                            if *num < 24 {
                                *num = 107;
                            }

                            debug!("note is now: {num}"); 
                        });
                        step.note
                    })
                })
            })
        });
    }

    pub fn start_playing(&mut self) {
        self.playing.store(true, Ordering::Relaxed);
        info!("am now playing sequence");
    }

    pub fn stop_playing(&mut self) {
        self.playing.store(false, Ordering::Relaxed);
        info!("have stoped playing sequence");
    }

    pub fn get_step(&self) -> usize {
        self.step_i.load(Ordering::Relaxed)
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn get_step_state(&self, channel_i: usize, step_i: usize) -> StepState {
        let section_i = self.section_i.load(Ordering::Relaxed);

        self.steps[section_i][channel_i].read().unwrap().steps[step_i]
    }

    pub fn get_section(&self) -> usize {
        self.section_i.load(Ordering::Relaxed).into()
    }

    pub fn get_bpm(&self) -> usize {
        self.bpm.load(Ordering::Relaxed).into()
    }
}

fn do_run_sequence(
    mixer: Mixer,
    steps: Arc<[Arc<[RwLock<StepSequence>]>]>,
    step_i: Arc<AtomicUsize>,
    section_i: Arc<AtomicUsize>,
    playing: Arc<AtomicBool>,
    bpm: Arc<AtomicUsize>,
) {
    let bpq = 48;
    // beats_per_quater_note / (2 * <distance from a quarter>)
    let sixteenth_pulse = bpq / 4;
    // setup sync pulse time
    let calc_wait_time = || {
        Duration::from_secs_f64({
            let bpm = bpm.load(Ordering::Relaxed) as f64;
            let bpq = bpq as f64;

            (60.0 / bpm) / bpq
        })
    };
    trace!("pusle time = {}", calc_wait_time().as_secs_f64());
    let should_play = || playing.load(Ordering::Relaxed);
    let mut note_offs: Vec<(u8, usize)> = Vec::with_capacity(N_CHANNELS * 4);
    let mut pulses = 0;
    let increment_step_i = || {
        let tmp_step_i = step_i.load(Ordering::Relaxed);
        step_i.store((tmp_step_i + 1) % N_STEPS, Ordering::Relaxed);
        step_i.load(Ordering::Relaxed)
    };
    let mut stop_notes = {
        let mut mixer = mixer.clone();

        move |note_offs: &mut Vec<(u8, usize)>| {
            for (note, channel_i) in note_offs.iter() {
                trace!("stopping note: {note}");
                mixer.stop_notes(vec![*note], *channel_i);
            }

            note_offs.clear();
        }
    };

    loop {
        if should_play() {
            if pulses == 0 {
                // happens first so that the step_i value is always the step thats playing
                let i = increment_step_i();
                trace!("playing step {i}");

                // play notes from the current step.
                for (channel_i, (mix_channel, steps)) in mixer
                    .channels
                    .iter()
                    .zip(steps[section_i.load(Ordering::Relaxed)].iter())
                    .enumerate()
                {
                    if let Ok(mut mix_channel) = mix_channel.write() {
                        if let Ok(steps) = steps.read() {
                            let step = steps.steps[i];
                            trace!("step[{i}]: {step:?}");

                            if let Some(sound_gen) = &mut mix_channel.sound_gen {
                                let mut events = Vec::with_capacity(8);

                                if let Some(note) = step.note {
                                    events.push(MidiEvent::note_on(
                                        note,
                                        step.velocity,
                                        step.channel,
                                        0,
                                    ));
                                    trace!("playing note: {note}, on channel: {channel_i}");

                                    note_offs.push((note, channel_i));
                                }

                                if step.pitch_bend != 0.0 {
                                    let bend_amt =
                                        step.pitch_bend * MidiEvent::PITCH_BEND_CENTER as f32;
                                    let bend_amt = MidiEvent::PITCH_BEND_CENTER + bend_amt as u16;
                                    let event = MidiEvent::pitch_bend(bend_amt, step.channel, 0);

                                    events.push(event);
                                }

                                if step.mod_whl > 0.0 {
                                    let mut value = (step.mod_whl * 127.0).round() as u8;

                                    if value > 127 {
                                        value = 127;
                                    }

                                    let event = MidiEvent::control_change(1, value, 0, 0);

                                    events.push(event);
                                }

                                for ctrl in [step.macro_1, step.macro_2, step.macro_3, step.macro_4]
                                {
                                    if let Some((cc, val)) = ctrl {
                                        let mut value = (val * 127.0).round() as u8;

                                        if value > 127 {
                                            value = 127;
                                        }

                                        let event = MidiEvent::control_change(cc, value, 0, 0);

                                        events.push(event);
                                    }
                                }

                                if !events.is_empty() {
                                    if let Err(e) = sound_gen.send_midi(&events) {
                                        error!("sending midi failed with error {e}");
                                    }
                                }
                            }
                        }
                    }
                }
            } else if pulses == sixteenth_pulse - 1 {
                stop_notes(&mut note_offs);
            } else {
                trace!("pulse count = {pulses}");
            }

            pulses = (pulses + 1) % sixteenth_pulse;
            sleep(calc_wait_time());
        } else if !note_offs.is_empty() {
            trace!("note_offs is not empty and stepper is not playing");
            stop_notes(&mut note_offs);

            // reset step_i and pulses
            step_i.store(0, Ordering::Relaxed);
            pulses = 0;
        } else {
            // do nothing bc we want playback start to be super responsive
        }
    }
}

#[cfg(test)]
mod test {
    use std::{thread::sleep, time::Duration};

    use rack::*;

    use crate::{N_CHANNELS, mixer::Mixer, step_sequencer::StepSequencer};

    #[test]
    fn audio_ouptut() {
        // env_logger::builder().format_timestamp(None).init();

        let (mixer, dev) = Mixer::new();
        let (mut seq, _audio_wrapper) = StepSequencer::new(mixer, dev);
        let chan = 0;

        for chan in 0..N_CHANNELS {
            seq.mixer.set_instrument(chan, "Wt Synth".into());
        }

        let on_events = vec![
            MidiEvent::note_on(60, 100, 0, 0), // Middle C (C4), velocity 100, channel 0
            MidiEvent::note_on(64, 100, 0, 0), // E4
            MidiEvent::note_on(67, 100, 0, 0), // G4
        ];
        // info!("instrument loaded");

        if let Ok(mut channel) = seq.mixer.channels[chan].write() {
            if let Some(sound_gen) = &mut channel.sound_gen {
                if let Err(e) = sound_gen.send_midi(&on_events) {
                    panic!("sending midi failed with error {e}");
                }
            } else {
                panic!("no sound generator");
            }
        } else {
            panic!("failed to write channel {chan}");
        }

        sleep(Duration::from_secs(5));

        if let Some(plugin) = &seq.mixer.channels[chan].read().unwrap().sound_gen {
            log::debug!("sound_gen = {:?}", plugin.info().name.clone());
        }

        // panic!("foobar");
    }
}
