# Dream Of DAW:

A DAW written in Rust and Python3 for retro-emulaiton handhelds running linux.

it allows for:
- running/loading VST3 plugings
- game controller controls
- 4 instrument plugins (on for each channel)
- 3 effects per channel
- 3 effects on the main outputs

## TODO

- [ ] design an sqlite schema for storing and recalling plugin information.
- [ ] a way to set macros based on plugin (recall from sqlite)
- [ ] top view (macros, pitch, & mod-wheel)
- [ ] hint menu-bar
- [ ] add a fucntion like do_single_press but will trigger its action once on press, and then "re-trigger" its action periodically when the button is held
- [ ] make steps poly-phonic or add more channels or make a polyphonic drum track
