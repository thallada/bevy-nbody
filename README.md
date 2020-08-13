# bevy-nbody

An N-body simulation in Rust using the [bevy](https://bevyengine.org) crate for rendering and [bigbang](https://docs.rs/bigbang/0.0.9/bigbang/) crate for the n-body calculations.

![screenshot](assets/nbody.png)
[youtube video](https://youtu.be/7_NheElcuu8)

## Install

Clone the repo, and run `cargo build --release`. The executable will be under `target/release/`.

## Usage

```
Usage: bevy-nbody [-n <num-bodies>] [-t <time-step>] [-w <width>] [-h <height>] [-s <scale>]

n-body simulation in bevy using bigbang

Options:
  -n, --num-bodies  number of bodies in the simulation
  -t, --time-step   granularity of simulation (how much each frame impacts
                    movement)
  -w, --width       initial width of spawned window
  -h, --height      initial height of spawned window
  -s, --scale       initial scale of view (bigger = more zoomed out)
  --help            display usage information
```

## Controls

|key | control|
|----|--------|
| R | reset the simulation |
| Left Click | hold and move mouse to pan the view | 
| Middle Click | hold and move mouse up and down to zoom in and out |
| Right Click | click on a body to focus the camera on that body |