pactl load-module module-null-sink sink_name=MLWS_sink sink_properties=device.description="MLWS_OUT"
pactl load-module module-null-sink sink_name=MIC_sink sink_properties=device.description="MIC_OUT"
pactl load-module module-loopback source=MLWS_sink.monitor sink=MIC_sink

pactl load-module module-loopback source=alsa_input.usb-Plantronics_Plantronics_Blackwire_3220_Series_CC1D0E1627BA4D178967E96DD3D37F21-00.analog-stereo sink=MIC_sink
pactl load-module module-loopback source=MLWS_sink.monitor sink=alsa_output.usb-Plantronics_Plantronics_Blackwire_3220_Series_CC1D0E1627BA4D178967E96DD3D37F21-00.iec958-stereo
pactl load-module module-remap-source source_name=MLWS_source master=MIC_sink.monitor