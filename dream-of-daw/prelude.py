import sounddevice as sd
from pedalboard import load_plugin

SAMPLE_RATE = 48000
BUFFER_SIZE = 128
# SYNTH_PLUGIN = "./Wt Synth.vst3"  # .dll, .vst3, .vst, .component
# .dll, .vst3, .vst, .component
SYNTH_PLUGIN = "/home/yogurt/.vst3/Wt Synth.vst3"
# print(f"plugin params = {instrument.parameters.keys()}")

# output_dev = AudioStream.output_device_names
# print(f"output dev: {output_dev}")
# Get the name of the default output device
output_dev = sd.query_devices()[0]
SAMPLE_RATE = output_dev['default_samplerate'] if output_dev['default_samplerate'] else SAMPLE_RATE

# print(f"Default output device name: {output_dev}")


print("before load")
instrument = load_plugin(SYNTH_PLUGIN)
print("plugin load")
