# soundboard
Mr Llama's Wonderful Soundboard (MLWS)

A sound board written in Rust

(Sound code taken from https://github.com/gamebooster/soundboard)

### providing virtual microphone on windows

1. download and install vb-audio virtual cable from https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack43.zip
2. set soundboard loopback device to `CABLE Input`
3. use applications with input (this is the virtual microphone) `CABLE Output`
