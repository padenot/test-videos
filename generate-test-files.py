#!/usr/bin/env python3

import os

DURATION = 3
RATE = 30

RESOLUTIONS = [
    "640x480",
    "1280x720",
    "1920x1080",
    "3840x2160",
    "7680x4320",
]

CODECS = [
    ("av1", "libsvtav1", ["webm", "mp4"], "-preset 11 -crf 35 -svtav1-params lp=2"),
    ("h264", "libx264", ["mp4"], "-preset ultrafast -threads 2"),
    (
        "h265",
        "libx265",
        ["mp4"],
        "-preset ultrafast -threads 2 -x265-params pools=2:frame-threads=1:log-level=error",
    ),
    ("vp8", "libvpx", ["webm"], "-deadline realtime -cpu-used 8 -threads 2"),
    ("vp9", "libvpx-vp9", ["webm", "mp4"], "-deadline realtime -cpu-used 8 -threads 2"),
]

RGB_PIXEL_FORMATS = ["gbrp", "gbrp10le", "gbrp12le", "gbrp14le", "gbrp16le"]
YUV_PIXEL_FORMATS = [
    "yuv420p",
    "yuv422p",
    "yuv444p",
    "yuv420p10",
    "yuv422p10",
    "yuv444p10",
    "yuv420p12",
    "yuv422p12",
    "yuv444p12",
]

# testsrc2 is a broad codec stress source: moving edges, color fields, and
# luma/chroma detail. rgbtestsrc and yuvtestsrc make component-plane mistakes
# obvious. SMPTE HD bars are the SDR broadcast sanity source.
SDR_SOURCES = [
    ("sdr-testsrc2", "testsrc2", RGB_PIXEL_FORMATS + YUV_PIXEL_FORMATS),
    ("sdr-rgbtestsrc", "rgbtestsrc", RGB_PIXEL_FORMATS),
    ("sdr-yuvtestsrc", "yuvtestsrc", YUV_PIXEL_FORMATS),
    ("sdr-smptehdbars", "smptehdbars", YUV_PIXEL_FORMATS),
]

HDR_TRANSFERS = ["pq", "hlg"]
HDR_RESOLUTIONS = ["1920x1080", "3840x2160", "7680x4320"]


def codec_supports_pixel_format(codec, pix_fmt):
    if codec == "av1" and pix_fmt not in ("yuv420p", "yuv420p10"):
        return False
    if codec == "vp8" and ("10" in pix_fmt or "12" in pix_fmt):
        return False
    if codec == "h264" and "12" in pix_fmt:
        return False
    return True


def output_name(source_name, resolution, codec_lib, pix_fmt, container):
    return f"{source_name}-{resolution}-{codec_lib}-{pix_fmt}.{container}"


def sdr_input_filter(source_filter, resolution):
    return f"{source_filter}=duration={DURATION}:size={resolution}:rate={RATE}"


def sdr_encode_command(source_filter, resolution, pix_fmt, codec_lib, codec_options, output):
    return (
        "ffmpeg -n -f lavfi "
        f"-i {sdr_input_filter(source_filter, resolution)} "
        "-color_primaries bt709 -color_trc bt709 -colorspace bt709 "
        f"-pix_fmt {pix_fmt} -c:v {codec_lib} {codec_options} {output}"
    )


def write_command(f, output, command):
    f.write(f"[ -f {output} ] || {command}\n")


def write_sdr_commands(f):
    f.write("# SDR vectors\n")
    for source_name, source_filter, pixel_formats in SDR_SOURCES:
        for codec, codec_lib, containers, codec_options in CODECS:
            for resolution in RESOLUTIONS:
                for pix_fmt in pixel_formats:
                    if not codec_supports_pixel_format(codec, pix_fmt):
                        continue
                    for container in containers:
                        output = output_name(
                            source_name, resolution, codec_lib, pix_fmt, container
                        )
                        write_command(
                            f,
                            output,
                            sdr_encode_command(
                                source_filter,
                                resolution,
                                pix_fmt,
                                codec_lib,
                                codec_options,
                                output,
                            ),
                        )
    f.write("\n")


def write_hdr_commands(f):
    f.write("# HDR BT.2111 vectors. Raw Y4M generation lives in Rust.\n")
    for resolution in HDR_RESOLUTIONS:
        for transfer in HDR_TRANSFERS:
            for codec_lib, container in (("libvpx-vp9", "webm"), ("libx265", "mp4")):
                f.write(
                    "[ -f bt2111-{transfer}-{resolution}-{codec_lib}-yuv420p10.{container} ] || "
                    "RESOLUTION={resolution} FRAMES=90 ./generate-bt2111.sh\n".format(
                        transfer=transfer,
                        resolution=resolution,
                        codec_lib=codec_lib,
                        container=container,
                    )
                )


def main():
    with open("commands.sh", "w") as f:
        f.write("#!/bin/sh -eu\n\n")
        write_sdr_commands(f)
        write_hdr_commands(f)

    os.chmod("commands.sh", 0o755)


if __name__ == "__main__":
    main()
