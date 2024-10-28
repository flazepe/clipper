# clipper

A simple ffmpeg wrapper for clipping videos.

## Example Usages

Combining multiple clips from one input

```
clipper -input input.mp4 -segment 2:00-2:30 -segment 5:12-5:20 -segment 6:11-6:17 output.mp4
clipper -i input.mp4 -s 2:00-2:30 -s 5:12-5:20 -s 6:11-6:17 output.mp4
```

Combining multiple clips from multiple inputs

```
clipper -i input1.mp4 -s 2:00-2:30 -s 5:12-5:20 -i input2.mp4 -s 1:15-1:25 -s 7:20-7:27 output.mp4
```

Selecting an audio track from input (these options work per input)

```
clipper -input input.mp4 -audio-track 1 -segment 2:00-2:30 -segment 5:12-5:20 output.mp4
clipper -input input.mp4 -at 1 -s 2:00-2:30 -s 5:12-5:20 output.mp4
```

Adding a fade transition between segments with optional duration in seconds (this option applies to all segments, regardless of their inputs)

```
clipper -i input.mp4 -s 2:00-2:30 -s 5:12-5:20 -fade output.mp4
clipper -i input.mp4 -s 2:00-2:30 -s 5:12-5:20 -f output.mp4

clipper -i input.mp4 -s 2:00-2:30 -s 5:12-5:20 -fade=1 output.mp4
clipper -i input.mp4 -s 2:00-2:30 -s 5:12-5:20 -f=1 output.mp4
```

Other options can be found by using the help command (`clipper -help`).
