# Test Videos

This repository vendors generators and scripts, not generated media.

Generated `*.mp4`, `*.webm`, and `*.y4m` files are disposable artifacts. They
are ignored by Git and can be recreated from the scripts in this repository.

## Vector Families

- `sdr-testsrc2-*`: broad SDR codec and pixel-format stress vectors generated
  from FFmpeg `testsrc2`.
- `sdr-rgbtestsrc-*`: SDR RGB component-order vectors generated from FFmpeg
  `rgbtestsrc`.
- `sdr-yuvtestsrc-*`: SDR YUV plane and subsampling vectors generated from
  FFmpeg `yuvtestsrc`.
- `sdr-smptehdbars-*`: SDR broadcast sanity vectors generated from FFmpeg
  `smptehdbars`.
- `bt2111-*`: HDR PQ/HLG BT.2111 vectors generated from the in-repository Rust
  `bt2111-gen` tool, then encoded by FFmpeg.

## Generate

Regenerate the command matrix:

```sh
./generate-test-files.py
```

Generate all vectors listed by the current matrix:

```sh
./commands.sh
```

Generate only BT.2111 HDR vectors:

```sh
RESOLUTION=1920x1080 ./generate-bt2111.sh
RESOLUTION=4k ./generate-bt2111.sh
RESOLUTION=8k FRAMES=1 ./generate-bt2111.sh
```

Remove generated media and build products:

```sh
./clean-generated.sh
```

Run the representative validation set in parallel:

```sh
JOBS=$(nproc) ./validate-generated-vectors.sh
```
