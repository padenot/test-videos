# BT.2111 Raw-Frame Generator Plan

Goal: build a clean, license-friendly BT.2111 generator from the ITU spec that
outputs raw/Y4M frames only. Encoding remains a separate step so the corpus can
exercise different encoders and decoders.

## Context

- We initially generated rough BT.2111-style VP9/WebM clips with FFmpeg drawing
  filters. Those were removed because they did not match the ITU layout.
- We then used `test-full-band/tfb-video` externally from `/tmp/tfb-video` as a
  build-time/reference tool.
- Its `BT2111Generator` produced BT.2111 frames, which were converted with
  ffmpeg into:
  - `bt2111-pq-1920x1080-libvpx-vp9-yuv420p10.webm`
  - `bt2111-hlg-1920x1080-libvpx-vp9-yuv420p10.webm`
- `BT2111-PROVENANCE.txt` documents that temporary provenance.
- Long-term, do not depend on or port GPL implementation logic. Use the ITU
  BT.2111 document as the normative source. `tfb-video` output can be used only
  as a temporary validation oracle.

## Proposed Tool

Create a small standalone Rust tool, tentatively `bt2111-gen`.

Inputs:

- `--transfer pq|hlg`
- `--resolution 1920x1080`
- `--frames N`
- `--output out.y4m`

Initial output:

- Y4M containing 10-bit 4:2:0 planar video, equivalent to `yuv420p10le`
- No codec dependency
- No muxing
- No HDR container metadata

Later optional output:

- Raw planar `.yuv`
- Other integer-scaled resolutions such as `3840x2160`

## Implementation Steps

1. Add the Rust tool.
   - Keep it standalone and small.
   - Prefer no heavy image dependencies unless they clearly reduce risk.
   - Generate one or more identical frames.

2. Implement frame primitives.
   - 10-bit Y, U, V planar buffers.
   - Rectangle fill helpers.
   - Horizontal luma ramp helper.
   - Chroma fill helpers for 4:2:0.
   - Ensure BT.2111 rectangles are even-aligned where chroma is written.

3. Implement the BT.2111 layout from the ITU document.
   - Start with exact 1920x1080 constants.
   - Support only integer scale factors from 1920x1080 initially.
   - Keep comments citing ITU figure/table names.
   - Avoid copying GPL code structure or comments.

4. Implement color math from specs.
   - BT.2020 RGB to limited-range YCbCr.
   - ST 2084/PQ transfer.
   - HLG transfer.
   - BT.709 RGB to XYZ to BT.2020 conversion for bottom BT.709 reference bars.
   - Clamp and round consistently.

5. Generate all BT.2111 regions.
   - Side grey fields.
   - Top full-intensity bars.
   - Dimmed bars.
   - Stair row.
   - Ramp row.
   - Bottom BT.709 bars.
   - Black regions.
   - PLUGE bars.
   - White block.
   - Omit overlay text unless required by the spec; avoiding text keeps the raw
     generator deterministic and font-free.

6. Add verification.
   - Unit tests for PQ known points.
   - Unit tests for HLG known points.
   - Unit tests for RGB to YCbCr known colors.
   - Pixel tests for representative coordinates in each BT.2111 region.
   - Optional temporary comparison against `tfb-video` Y4M output, but do not
     copy implementation details.

7. Integrate with corpus encoding.
   - Generate `bt2111-pq-1920x1080-yuv420p10.y4m`.
   - Generate `bt2111-hlg-1920x1080-yuv420p10.y4m`.
   - Add a separate encoding script, or extend `commands.sh`, to encode these
     Y4M files with desired encoders:
     - VP9/WebM
     - AV1/WebM or MP4
     - H.264 where useful
     - H.265/HEVC where useful
   - Keep raw generation separate from codec generation.

8. Update website and docs.
   - `test-videos.html` already discovers `.mp4` and `.webm` dynamically.
   - If raw `.y4m` files should be listed, extend discovery to include `.y4m`.
   - Replace or update `BT2111-PROVENANCE.txt` once the Rust generator becomes
     authoritative.

## Licensing Rules

- Do not vendor `test-full-band/tfb-video`.
- Do not port GPL code structure.
- Do not copy GPL constants/comments if they are not directly traceable to ITU
  BT.2111 or another permissive/normative source.
- Use ITU BT.2111 as the source of truth.
- Use `tfb-video` output only as validation data during development.

## Current Useful Commands

Current generated BT.2111-derived WebM files can be inspected with:

```sh
ffprobe -v error -select_streams v:0 \
  -show_entries stream=codec_name,width,height,pix_fmt,color_range,color_space,color_transfer,color_primaries \
  -show_entries format=duration \
  -of default=noprint_wrappers=1 \
  bt2111-pq-1920x1080-libvpx-vp9-yuv420p10.webm

ffprobe -v error -select_streams v:0 \
  -show_entries stream=codec_name,width,height,pix_fmt,color_range,color_space,color_transfer,color_primaries \
  -show_entries format=duration \
  -of default=noprint_wrappers=1 \
  bt2111-hlg-1920x1080-libvpx-vp9-yuv420p10.webm
```

Expected metadata:

- PQ: `vp9`, `1920x1080`, `yuv420p10le`, `bt2020nc/bt2020/smpte2084`,
  duration `3.000000`
- HLG: `vp9`, `1920x1080`, `yuv420p10le`, `bt2020nc/bt2020/arib-std-b67`,
  duration `3.000000`
