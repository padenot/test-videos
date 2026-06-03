#!/usr/bin/env python3

import os

codec = [
  ("av1", "webm"),
  ("av1", "mp4"),
  ("h264", "mp4"),
  ("h265", "mp4"),
  ("vp8", "webm"),
  ("vp9", "webm"),
  ("vp9", "mp4")
]

codectolib = {
    "vp8": "libvpx",
    "vp9": "libvpx-vp9",
    "av1": "libaom-av1",
    "h264": "libx264",
    "h265": "libx265"
}

# 8 bits is implied in ffmpeg pixel format
bit_depth = [ "", 10, 12 ]
subsampling  = [ 420, 422, 444 ]

pixel_format = ["gbrp", "gbrp10le", "gbrp12le", "gbrp14le", "gbrp16le"]
for bd in bit_depth:
    for subs in subsampling:
        pixel_format.append("yuv{}p{}".format(subs, bd))

resolution=["640x480", "1280x720", "1920x1080", "3840x2160", "7680x4320"]

name_pattern = "pattern-{}-{}-{}.{}"

# -n to always refuse overwriting to allow rerunning the script
fmt = "ffmpeg -n -f lavfi -i testsrc=duration=3:size={}:rate=30 -pix_fmt {} -c:v {} {}"

def write_command(f, name, command):
    f.write("[ -f {} ] || ".format(name))
    f.write(command)
    f.write("\n")


with open("commands.sh", "w") as f:
    f.write("#!/bin/sh -xe\n\n")
    for (codec, container) in codec:
        for res in resolution:
            for format in pixel_format:
                if "libvpx" in codectolib[codec] and format == "rgba":
                    # no support for rgba in libvpx
                    continue
                if codec == "vp8" and ("10" in format or "12" in format):
                    # libvpx-vp8 doesn't support 10 and 12 bits
                    continue
                if codec == "h264" and ("12" in format or format == "rgba"):
                    continue
                name = name_pattern.format(res, codectolib[codec], format, container)
                print(name)
                write_command(f, name, fmt.format(res, format, codectolib[codec], name))

os.chmod("commands.sh", 0o755)
