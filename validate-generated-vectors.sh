#!/bin/sh -eu

jobs="${JOBS:-$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}"
tmpdir="${TMPDIR:-/tmp}/test-videos-validation-$$"

mkdir -p "$tmpdir"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

python3 -m py_compile generate-test-files.py
sh -n commands.sh
cargo test
cargo build --release

cat > "$tmpdir/ffmpeg-commands" <<EOF
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=1920x1080:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p -c:v libx264 "$tmpdir/sdr-testsrc2-h264-yuv420p.mp4"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=1920x1080:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv444p10 -c:v libx265 -x265-params log-level=error "$tmpdir/sdr-testsrc2-h265-yuv444p10.mp4"
ffmpeg -y -v error -f lavfi -i rgbtestsrc=duration=0.2:size=1920x1080:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt gbrp10le -c:v libaom-av1 "$tmpdir/sdr-rgbtestsrc-av1-gbrp10le.webm"
ffmpeg -y -v error -f lavfi -i yuvtestsrc=duration=0.2:size=1920x1080:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv422p10 -c:v libvpx-vp9 "$tmpdir/sdr-yuvtestsrc-vp9-yuv422p10.webm"
ffmpeg -y -v error -f lavfi -i smptehdbars=duration=0.2:size=1920x1080:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p10 -c:v libx265 -x265-params log-level=error "$tmpdir/sdr-smptehdbars-h265-yuv420p10.mp4"
ffmpeg -y -v error -f lavfi -i testsrc2=duration=0.2:size=3840x2160:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p -c:v libvpx "$tmpdir/sdr-testsrc2-vp8-4k-yuv420p.webm"
ffmpeg -y -v error -f lavfi -i yuvtestsrc=duration=0.1:size=7680x4320:rate=30 -color_primaries bt709 -color_trc bt709 -colorspace bt709 -pix_fmt yuv420p10 -c:v libvpx-vp9 "$tmpdir/sdr-yuvtestsrc-vp9-8k-yuv420p10.webm"
./target/release/bt2111-gen --transfer pq --resolution 1920x1080 --frames 1 --output "$tmpdir/bt2111-pq-1920x1080-yuv420p10.y4m"
./target/release/bt2111-gen --transfer hlg --resolution 3840x2160 --frames 1 --output "$tmpdir/bt2111-hlg-3840x2160-yuv420p10.y4m"
./target/release/bt2111-gen --transfer pq --resolution 7680x4320 --frames 1 --output "$tmpdir/bt2111-pq-7680x4320-yuv420p10.y4m"
EOF

xargs -P "$jobs" -I {} sh -c '{}' < "$tmpdir/ffmpeg-commands"
