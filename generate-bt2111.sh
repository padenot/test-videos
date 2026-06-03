#!/bin/sh -eu

RESOLUTION="${RESOLUTION:-1920x1080}"
FRAMES="${FRAMES:-90}"

case "$RESOLUTION" in
  fhd|1080p|2k|1920x1080) SIZE=1920x1080 ;;
  uhd|4k|3840x2160) SIZE=3840x2160 ;;
  8k|7680x4320) SIZE=7680x4320 ;;
  *x*) SIZE="$RESOLUTION" ;;
  *) echo "unsupported RESOLUTION=$RESOLUTION" >&2; exit 1 ;;
esac

cargo run --release -- --transfer pq --resolution "$SIZE" --frames "$FRAMES" \
  --output "bt2111-pq-$SIZE-yuv420p10.y4m"
cargo run --release -- --transfer hlg --resolution "$SIZE" --frames "$FRAMES" \
  --output "bt2111-hlg-$SIZE-yuv420p10.y4m"

ffmpeg -n -i "bt2111-pq-$SIZE-yuv420p10.y4m" \
  -vf setparams=color_primaries=bt2020:color_trc=smpte2084:colorspace=bt2020nc:range=tv \
  -c:v libvpx-vp9 -pix_fmt yuv420p10le \
  "bt2111-pq-$SIZE-libvpx-vp9-yuv420p10.webm"

ffmpeg -n -i "bt2111-hlg-$SIZE-yuv420p10.y4m" \
  -vf setparams=color_primaries=bt2020:color_trc=arib-std-b67:colorspace=bt2020nc:range=tv \
  -c:v libvpx-vp9 -pix_fmt yuv420p10le \
  "bt2111-hlg-$SIZE-libvpx-vp9-yuv420p10.webm"
