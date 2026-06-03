#!/bin/sh -eu

jobs="${JOBS:-$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}"
tmpdir="${TMPDIR:-/tmp}/test-videos-validation-$$"

mkdir -p "$tmpdir"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

python3 -m py_compile generate-test-files.py
sh -n commands.sh
cargo test
cargo build --release

bt709="setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709:range=tv"

cat > "$tmpdir/ffmpeg-commands" <<EOF
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=1920x1080:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p -c:v libx264 -preset medium -crf 18 -threads 2 "$tmpdir/sdr-testsrc2-h264-yuv420p.mp4"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=1920x1080:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv444p10 -c:v libx265 -preset medium -crf 20 -threads 2 -x265-params pools=2:frame-threads=1:log-level=error "$tmpdir/sdr-testsrc2-h265-yuv444p10.mp4"
ffmpeg -y -v error -f lavfi -i yuvtestsrc=duration=0.2:size=1920x1080:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p10 -c:v libsvtav1 -preset 8 -crf 24 -svtav1-params lp=4 "$tmpdir/sdr-yuvtestsrc-av1-yuv420p10.webm"
ffmpeg -y -v error -f lavfi -i yuvtestsrc=duration=0.2:size=1920x1080:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv422p10 -c:v libvpx-vp9 -deadline good -cpu-used 4 -crf 24 -b:v 0 -threads 2 "$tmpdir/sdr-yuvtestsrc-vp9-yuv422p10.webm"
ffmpeg -y -v error -f lavfi -i smptehdbars=duration=0.2:size=1920x1080:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p10 -c:v libx265 -preset medium -crf 20 -threads 2 -x265-params pools=2:frame-threads=1:log-level=error "$tmpdir/sdr-smptehdbars-h265-yuv420p10.mp4"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=3840x2160:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p -c:v libvpx -deadline good -cpu-used 4 -crf 24 -b:v 0 -threads 2 "$tmpdir/sdr-testsrc2-vp8-4k-yuv420p.webm"
ffmpeg -y -v error -f lavfi -i yuvtestsrc=duration=0.1:size=7680x4320:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p10 -c:v libvpx-vp9 -deadline good -cpu-used 4 -crf 24 -b:v 0 -threads 2 "$tmpdir/sdr-yuvtestsrc-vp9-8k-yuv420p10.webm"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=854x480:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p -c:v libx264 -profile:v baseline -preset medium -crf 18 -threads 2 "$tmpdir/sdr-testsrc2-h264-baseline-yuv420p.mp4"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=854x480:rate=30 -vf $bt709 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv444p10 -c:v libvpx-vp9 -profile:v 3 -deadline good -cpu-used 4 -crf 24 -b:v 0 -threads 2 "$tmpdir/sdr-testsrc2-vp9-profile3-yuv444p10.webm"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=854x480:rate=30 -vf $bt709,format=yuva420p,fade=t=in:st=0:d=0.2:alpha=1 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -c:v libvpx -auto-alt-ref 0 -deadline good -cpu-used 4 -crf 24 -b:v 0 -threads 2 "$tmpdir/sdr-testsrc2-vp8-alpha-yuva420p.webm"
./target/release/bt2111-gen --transfer pq --resolution 1920x1080 --frames 1 --output "$tmpdir/bt2111-pq-1920x1080-yuv420p10.y4m"
./target/release/bt2111-gen --transfer hlg --resolution 3840x2160 --frames 1 --output "$tmpdir/bt2111-hlg-3840x2160-yuv420p10.y4m"
./target/release/bt2111-gen --transfer pq --resolution 7680x4320 --frames 1 --output "$tmpdir/bt2111-pq-7680x4320-yuv420p10.y4m"
EOF

xargs -P "$jobs" -I {} sh -c '{}' < "$tmpdir/ffmpeg-commands"

ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=854x480:rate=30 \
  -vf "$bt709" -color_primaries bt709 -color_trc bt709 -colorspace bt709 \
  -pix_fmt yuv420p -c:v libx264 -preset medium -crf 18 -threads 2 \
  "$tmpdir/rotation-base.mp4"
ffmpeg -y -v error -display_rotation 90 -i "$tmpdir/rotation-base.mp4" \
  -c copy "$tmpdir/sdr-testsrc2-h264-rotate90-yuv420p.mp4"
