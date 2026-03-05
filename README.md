# Collector Controller

egui program made to control the collector for yarn electrospinning

The collector we use in the lab has an arduino uno connected to a DM332T set at 400 pulses/rev

PUL=pin 7
DIR=pin 6

The program changes RPM and direction. Probably shouldnt change direction tho lol

The program does NOT change the default values for when the arduino stops/starts again

See the arduino.ino code for info on what is running on the arduino exactly

## Running

```
cargo run
```

Alternatively, just take an exe from the Releases and run that
