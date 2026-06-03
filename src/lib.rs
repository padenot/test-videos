use std::io::{self, Write};

const BASE_WIDTH: usize = 1920;
const BASE_HEIGHT: usize = 1080;
#[cfg(test)]
const TEST_NON_INTEGER_SCALE_WIDTH: usize = 1280;
#[cfg(test)]
const TEST_NON_INTEGER_SCALE_HEIGHT: usize = 720;
#[cfg(test)]
const UHD_WIDTH: usize = 3840;
#[cfg(test)]
const UHD_HEIGHT: usize = 2160;
#[cfg(test)]
const EIGHT_K_WIDTH: usize = 7680;
#[cfg(test)]
const EIGHT_K_HEIGHT: usize = 4320;
const LIMITED_BLACK: u16 = 64;
const LIMITED_WHITE: u16 = 940;
const CHROMA_NEUTRAL: u16 = 512;
const CODE_MIN_10BIT: u16 = 0;
const CODE_MAX_10BIT: u16 = 1023;

const LIMITED_MIN: f64 = LIMITED_BLACK as f64;
const LIMITED_LUMA_RANGE: f64 = 876.0;
const LIMITED_CHROMA_RANGE: f64 = 896.0;
const CHROMA_CENTER: f64 = CHROMA_NEUTRAL as f64;

const FULL_BAR_CODE: u16 = LIMITED_WHITE;
const HLG_MAIN_BAR_CODE: u16 = 721;
const PQ_MAIN_BAR_CODE: u16 = 572;
const GREY_40_CODE: u16 = 414;
const RAMP_MIN_CODE: u16 = 4;
const RAMP_MAX_CODE: u16 = 1019;
const RAMP_OFFSET_2K: i32 = 206;
const PLUGE_MINUS_2_CODE: u16 = 46;
const PLUGE_PLUS_2_CODE: u16 = 82;
const PLUGE_PLUS_4_CODE: u16 = 99;

const CENTRAL_WIDTH_2K: usize = 1440;
const SIDE_FIELD_WIDTH_2K: usize = 240;
const BAR_WIDE_2K: usize = 206;
const BAR_NARROW_2K: usize = 204;
const STAIR_WIDTHS_2K: [usize; 13] = [
    206, 103, 103, 103, 103, 102, 102, 103, 103, 103, 103, 103, 103,
];

const BOTTOM_BLACK_LEFT_WIDTH_2K: usize = 136;
const PLUGE_BAR_WIDTH_2K: usize = 70;
const PLUGE_BLACK_GAP_WIDTH_2K: usize = 68;
const BOTTOM_BLACK_MIDDLE_WIDTH_2K: usize = 238;
const BOTTOM_WHITE_WIDTH_2K: usize = 438;
const BOTTOM_BLACK_RIGHT_WIDTH_2K: usize = 282;

const TOP_BAR_HEIGHT_2K: usize = 90;
const MAIN_BAR_HEIGHT_2K: usize = 540;
const STAIR_Y_2K: usize = 630;
const RAMP_Y_2K: usize = 720;
const BOTTOM_Y_2K: usize = 810;
const ROW_HEIGHT_2K: usize = 90;
const BOTTOM_HEIGHT_2K: usize = 270;

const STAIR_CODES: [u16; 13] = [
    RAMP_MIN_CODE,
    LIMITED_BLACK,
    152,
    239,
    327,
    GREY_40_CODE,
    502,
    590,
    677,
    765,
    852,
    LIMITED_WHITE,
    RAMP_MAX_CODE,
];

// Final 10-bit BT.2020 YCbCr values for the bottom BT.709 reference patches.
const HLG_BT709_LEFT: [Ycbcr; 3] = [
    Ycbcr {
        y: 694,
        cb: 307,
        cr: 526,
    },
    Ycbcr {
        y: 664,
        cb: 541,
        cr: 424,
    },
    Ycbcr {
        y: 631,
        cb: 330,
        cr: 429,
    },
];
const HLG_BT709_RIGHT: [Ycbcr; 3] = [
    Ycbcr {
        y: 406,
        cb: 674,
        cr: 681,
    },
    Ycbcr {
        y: 360,
        cb: 405,
        cr: 705,
    },
    Ycbcr {
        y: 201,
        cb: 784,
        cr: 530,
    },
];
const PQ_BT709_LEFT: [Ycbcr; 3] = [
    Ycbcr {
        y: 559,
        cb: 415,
        cr: 518,
    },
    Ycbcr {
        y: 544,
        cb: 526,
        cr: 470,
    },
    Ycbcr {
        y: 529,
        cb: 424,
        cr: 474,
    },
];
const PQ_BT709_RIGHT: [Ycbcr; 3] = [
    Ycbcr {
        y: 419,
        cb: 591,
        cr: 593,
    },
    Ycbcr {
        y: 392,
        cb: 438,
        cr: 608,
    },
    Ycbcr {
        y: 277,
        cb: 667,
        cr: 540,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transfer {
    Pq,
    Hlg,
}

#[derive(Clone, Copy, Debug)]
pub struct GeneratorOptions {
    pub transfer: Transfer,
    pub width: usize,
    pub height: usize,
    pub frames: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ycbcr {
    pub y: u16,
    pub cb: u16,
    pub cr: u16,
}

#[derive(Clone)]
pub struct Frame {
    width: usize,
    y: Vec<u16>,
    u: Vec<u16>,
    v: Vec<u16>,
}

impl Frame {
    fn new(width: usize, height: usize) -> Self {
        let luma = width * height;
        let chroma = (width / 2) * (height / 2);
        Self {
            width,
            y: vec![LIMITED_BLACK; luma],
            u: vec![CHROMA_NEUTRAL; chroma],
            v: vec![CHROMA_NEUTRAL; chroma],
        }
    }

    pub fn pixel(&self, x: usize, y: usize) -> Ycbcr {
        let chroma_index = (y / 2) * (self.width / 2) + (x / 2);
        Ycbcr {
            y: self.y[y * self.width + x],
            cb: self.u[chroma_index],
            cr: self.v[chroma_index],
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Ycbcr) {
        for yy in rect.y..rect.y + rect.h {
            let row = yy * self.width;
            for xx in rect.x..rect.x + rect.w {
                self.y[row + xx] = color.y;
            }
        }

        let chroma_width = self.width / 2;
        let x0 = rect.x / 2;
        let y0 = rect.y / 2;
        let x1 = (rect.x + rect.w) / 2;
        let y1 = (rect.y + rect.h) / 2;
        for yy in y0..y1 {
            let row = yy * chroma_width;
            for xx in x0..x1 {
                self.u[row + xx] = color.cb;
                self.v[row + xx] = color.cr;
            }
        }
    }

    fn write_y4m_frame<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(b"FRAME\n")?;
        write_plane(writer, &self.y)?;
        write_plane(writer, &self.u)?;
        write_plane(writer, &self.v)
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[derive(Clone, Copy)]
struct RgbCode {
    r: u16,
    g: u16,
    b: u16,
}

pub fn write_y4m<W: Write>(writer: &mut W, options: GeneratorOptions) -> io::Result<()> {
    let frame = generate_frame(options)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;
    writeln!(
        writer,
        "YUV4MPEG2 W{} H{} F30:1 Ip A0:0 C420p10 XYSCSS=420P10",
        options.width, options.height
    )?;
    for _ in 0..options.frames {
        frame.write_y4m_frame(writer)?;
    }
    Ok(())
}

pub fn generate_frame(options: GeneratorOptions) -> Result<Frame, String> {
    validate_options(options)?;

    let scale = native_scale(options);
    let dims = Layout::new(scale);
    let levels = Levels::for_transfer(options.transfer);
    let mut frame = Frame::new(options.width, options.height);

    // ITU-R BT.2111-2 Fig. 1/Fig. 2: side grey fields and central colour bars.
    frame.fill_rect(dims.left_side_top(), ycbcr_from_rgb(levels.grey_40));
    frame.fill_rect(dims.right_side_top(), ycbcr_from_rgb(levels.grey_40));
    fill_bars(&mut frame, dims.top_bar_row(), &levels.full_bars);
    fill_bars(&mut frame, dims.main_bar_row(), &levels.main_bars);

    // ITU-R BT.2111-2 Fig. 1/Fig. 2: stair row from -7% to 109%.
    frame.fill_rect(dims.left_stair_cap(), ycbcr_from_rgb(levels.main_bars[0]));
    frame.fill_rect(dims.right_stair_cap(), ycbcr_from_rgb(levels.main_bars[0]));
    fill_steps(&mut frame, dims.stair_row(), &levels.steps);

    // ITU-R BT.2111-2 Fig. 5/Table 5: narrow-range ramp.
    fill_ramp(&mut frame, &dims, &levels);

    // ITU-R BT.2111-2 Fig. 1/Fig. 2: bottom BT.709 reference bars and PLUGE.
    fill_bottom(&mut frame, &dims, &levels);

    Ok(frame)
}

fn validate_options(options: GeneratorOptions) -> Result<(), String> {
    if options.frames == 0 {
        return Err("frames must be greater than zero".to_string());
    }
    if options.width < BASE_WIDTH || options.height < BASE_HEIGHT {
        return Err(format!(
            "native generation starts at {}x{}; generate lower resolutions by resampling",
            BASE_WIDTH, BASE_HEIGHT
        ));
    }
    if options.width % BASE_WIDTH != 0 || options.height % BASE_HEIGHT != 0 {
        return Err(format!(
            "only integer scales of {}x{} are supported",
            BASE_WIDTH, BASE_HEIGHT
        ));
    }
    if native_scale(options) != options.height / BASE_HEIGHT {
        return Err(format!(
            "width and height must use the same integer scale from {}x{}",
            BASE_WIDTH, BASE_HEIGHT
        ));
    }
    if options.width % 2 != 0 || options.height % 2 != 0 {
        return Err("4:2:0 output requires even dimensions".to_string());
    }
    Ok(())
}

fn native_scale(options: GeneratorOptions) -> usize {
    options.width / BASE_WIDTH
}

fn fill_bars(frame: &mut Frame, rect: Rect, colors: &[RgbCode; 7]) {
    let widths = bar_widths(rect.w);
    let mut x = rect.x;
    for (width, rgb) in widths.into_iter().zip(colors) {
        frame.fill_rect(
            Rect {
                x,
                y: rect.y,
                w: width,
                h: rect.h,
            },
            ycbcr_from_rgb(*rgb),
        );
        x += width;
    }
}

fn fill_steps(frame: &mut Frame, rect: Rect, levels: &[RgbCode; 13]) {
    let mut x = rect.x;
    let scale = rect.w / CENTRAL_WIDTH_2K;
    for (index, rgb) in levels.iter().enumerate() {
        let width = STAIR_WIDTHS_2K[index] * scale;
        frame.fill_rect(
            Rect {
                x,
                y: rect.y,
                w: width,
                h: rect.h,
            },
            ycbcr_from_rgb(*rgb),
        );
        x += width;
    }
}

fn fill_ramp(frame: &mut Frame, dims: &Layout, levels: &Levels) {
    // ITU-R BT.2111-2 Fig. 5/Table 5, 2K/10-bit narrow-range ramp.
    let y = dims.ramp_y;
    let h = dims.row_h;

    frame.fill_rect(
        Rect {
            x: 0,
            y,
            w: dims.side_field_w,
            h,
        },
        ycbcr_from_rgb(levels.black_0),
    );
    for xx in dims.side_field_w..dims.width {
        let value = RAMP_MAX_CODE as i32 + (xx / dims.scale) as i32 + RAMP_OFFSET_2K
            - (dims.width / dims.scale) as i32;
        frame.fill_rect(
            Rect { x: xx, y, w: 1, h },
            Ycbcr {
                y: clamp10(value.clamp(RAMP_MIN_CODE as i32, RAMP_MAX_CODE as i32)),
                cb: CHROMA_NEUTRAL,
                cr: CHROMA_NEUTRAL,
            },
        );
    }
}

fn fill_bottom(frame: &mut Frame, dims: &Layout, levels: &Levels) {
    let y = dims.bottom_y;
    let h = dims.bottom_h;
    let mut x = 0;
    for yuv in levels.bt709_left {
        frame.fill_rect(
            Rect {
                x,
                y,
                w: dims.bt709_patch_w,
                h,
            },
            yuv,
        );
        x += dims.bt709_patch_w;
    }

    frame.fill_rect(
        Rect {
            x,
            y,
            w: dims.bottom_black_left_w,
            h,
        },
        ycbcr_from_rgb(levels.black_0),
    );
    x += dims.bottom_black_left_w;

    for (w, rgb) in [
        (dims.pluge_bar_w, levels.black_minus_2),
        (dims.pluge_black_gap_w, levels.black_0),
        (dims.pluge_bar_w, levels.black_plus_2),
        (dims.pluge_black_gap_w, levels.black_0),
        (dims.pluge_bar_w, levels.black_plus_4),
    ] {
        frame.fill_rect(Rect { x, y, w, h }, ycbcr_from_rgb(rgb));
        x += w;
    }

    frame.fill_rect(
        Rect {
            x,
            y,
            w: dims.bottom_black_middle_w,
            h,
        },
        ycbcr_from_rgb(levels.black_0),
    );
    x += dims.bottom_black_middle_w;
    frame.fill_rect(
        Rect {
            x,
            y,
            w: dims.bottom_white_w,
            h,
        },
        ycbcr_from_rgb(levels.main_bars[0]),
    );
    x += dims.bottom_white_w;
    frame.fill_rect(
        Rect {
            x,
            y,
            w: dims.bottom_black_right_w,
            h,
        },
        ycbcr_from_rgb(levels.black_0),
    );
    x += dims.bottom_black_right_w;

    for yuv in levels.bt709_right {
        frame.fill_rect(
            Rect {
                x,
                y,
                w: dims.bt709_patch_w,
                h,
            },
            yuv,
        );
        x += dims.bt709_patch_w;
    }
}

fn bar_widths(total_width: usize) -> [usize; 7] {
    let scale = total_width / CENTRAL_WIDTH_2K;
    let wide = BAR_WIDE_2K * scale;
    let narrow = BAR_NARROW_2K * scale;
    [wide, wide, wide, narrow, wide, wide, wide]
}

fn write_plane<W: Write>(writer: &mut W, plane: &[u16]) -> io::Result<()> {
    for value in plane {
        writer.write_all(&value.to_le_bytes())?;
    }
    Ok(())
}

fn ycbcr_from_rgb(rgb: RgbCode) -> Ycbcr {
    let r = limited_code_to_float(rgb.r);
    let g = limited_code_to_float(rgb.g);
    let b = limited_code_to_float(rgb.b);

    // Rec. ITU-R BT.2020 non-constant-luminance coefficients.
    let kr = 0.2627;
    let kb = 0.0593;
    let kg = 1.0 - kr - kb;

    let y = kr * r + kg * g + kb * b;
    let cb = (b - y) / (2.0 * (1.0 - kb));
    let cr = (r - y) / (2.0 * (1.0 - kr));

    Ycbcr {
        y: limited_luma_to_code(y),
        cb: limited_chroma_to_code(cb),
        cr: limited_chroma_to_code(cr),
    }
}

fn limited_code_to_float(code: u16) -> f64 {
    (code as f64 - LIMITED_MIN) / LIMITED_LUMA_RANGE
}

fn limited_luma_to_code(value: f64) -> u16 {
    clamp10((LIMITED_MIN + LIMITED_LUMA_RANGE * value).round() as i32)
}

fn limited_chroma_to_code(value: f64) -> u16 {
    clamp10((CHROMA_CENTER + LIMITED_CHROMA_RANGE * value).round() as i32)
}

fn clamp10(value: i32) -> u16 {
    value.clamp(CODE_MIN_10BIT as i32, CODE_MAX_10BIT as i32) as u16
}

pub fn pq_oetf(luminance_nits: f64) -> f64 {
    let m1 = 2610.0 / 16384.0;
    let m2 = 2523.0 / 32.0;
    let c1 = 3424.0 / 4096.0;
    let c2 = 2413.0 / 128.0;
    let c3 = 2392.0 / 128.0;
    let y = (luminance_nits / 10000.0).max(0.0).powf(m1);
    ((c1 + c2 * y) / (1.0 + c3 * y)).powf(m2)
}

pub fn hlg_oetf(scene_linear: f64) -> f64 {
    let a: f64 = 0.17883277;
    let b = 1.0 - 4.0 * a;
    let c = 0.5 - a * (4.0_f64 * a).ln();
    if scene_linear <= 1.0 / 12.0 {
        (3.0 * scene_linear).sqrt()
    } else {
        a * (12.0 * scene_linear - b).ln() + c
    }
}

struct Levels {
    full_bars: [RgbCode; 7],
    main_bars: [RgbCode; 7],
    grey_40: RgbCode,
    steps: [RgbCode; 13],
    bt709_left: [Ycbcr; 3],
    bt709_right: [Ycbcr; 3],
    black_0: RgbCode,
    black_minus_2: RgbCode,
    black_plus_2: RgbCode,
    black_plus_4: RgbCode,
}

impl Levels {
    fn for_transfer(transfer: Transfer) -> Self {
        match transfer {
            Transfer::Hlg => Self {
                full_bars: bars(FULL_BAR_CODE),
                main_bars: bars(HLG_MAIN_BAR_CODE),
                grey_40: grey(GREY_40_CODE),
                steps: grey_steps(STAIR_CODES),
                bt709_left: HLG_BT709_LEFT,
                bt709_right: HLG_BT709_RIGHT,
                black_0: grey(LIMITED_BLACK),
                black_minus_2: grey(PLUGE_MINUS_2_CODE),
                black_plus_2: grey(PLUGE_PLUS_2_CODE),
                black_plus_4: grey(PLUGE_PLUS_4_CODE),
            },
            Transfer::Pq => Self {
                full_bars: bars(FULL_BAR_CODE),
                main_bars: bars(PQ_MAIN_BAR_CODE),
                grey_40: grey(GREY_40_CODE),
                steps: grey_steps(STAIR_CODES),
                bt709_left: PQ_BT709_LEFT,
                bt709_right: PQ_BT709_RIGHT,
                black_0: grey(LIMITED_BLACK),
                black_minus_2: grey(PLUGE_MINUS_2_CODE),
                black_plus_2: grey(PLUGE_PLUS_2_CODE),
                black_plus_4: grey(PLUGE_PLUS_4_CODE),
            },
        }
    }
}

fn bars(high: u16) -> [RgbCode; 7] {
    [
        RgbCode {
            r: high,
            g: high,
            b: high,
        },
        RgbCode {
            r: high,
            g: high,
            b: LIMITED_BLACK,
        },
        RgbCode {
            r: LIMITED_BLACK,
            g: high,
            b: high,
        },
        RgbCode {
            r: LIMITED_BLACK,
            g: high,
            b: LIMITED_BLACK,
        },
        RgbCode {
            r: high,
            g: LIMITED_BLACK,
            b: high,
        },
        RgbCode {
            r: high,
            g: LIMITED_BLACK,
            b: LIMITED_BLACK,
        },
        RgbCode {
            r: LIMITED_BLACK,
            g: LIMITED_BLACK,
            b: high,
        },
    ]
}

fn grey(value: u16) -> RgbCode {
    RgbCode {
        r: value,
        g: value,
        b: value,
    }
}

fn grey_steps(values: [u16; 13]) -> [RgbCode; 13] {
    values.map(grey)
}

struct Layout {
    scale: usize,
    side_field_w: usize,
    bottom_black_left_w: usize,
    pluge_bar_w: usize,
    pluge_black_gap_w: usize,
    bottom_black_middle_w: usize,
    bottom_white_w: usize,
    bottom_black_right_w: usize,
    bt709_patch_w: usize,
    top_h: usize,
    main_h: usize,
    stair_y: usize,
    ramp_y: usize,
    bottom_y: usize,
    row_h: usize,
    bottom_h: usize,
    width: usize,
}

impl Layout {
    fn new(scale: usize) -> Self {
        let side_field_w = SIDE_FIELD_WIDTH_2K * scale;
        Self {
            scale,
            side_field_w,
            bottom_black_left_w: BOTTOM_BLACK_LEFT_WIDTH_2K * scale,
            pluge_bar_w: PLUGE_BAR_WIDTH_2K * scale,
            pluge_black_gap_w: PLUGE_BLACK_GAP_WIDTH_2K * scale,
            bottom_black_middle_w: BOTTOM_BLACK_MIDDLE_WIDTH_2K * scale,
            bottom_white_w: BOTTOM_WHITE_WIDTH_2K * scale,
            bottom_black_right_w: BOTTOM_BLACK_RIGHT_WIDTH_2K * scale,
            bt709_patch_w: side_field_w / 3,
            top_h: TOP_BAR_HEIGHT_2K * scale,
            main_h: MAIN_BAR_HEIGHT_2K * scale,
            stair_y: STAIR_Y_2K * scale,
            ramp_y: RAMP_Y_2K * scale,
            bottom_y: BOTTOM_Y_2K * scale,
            row_h: ROW_HEIGHT_2K * scale,
            bottom_h: BOTTOM_HEIGHT_2K * scale,
            width: BASE_WIDTH * scale,
        }
    }

    fn central_x(&self) -> usize {
        self.side_field_w
    }

    fn central_w(&self) -> usize {
        self.width - 2 * self.side_field_w
    }

    fn left_side_top(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.side_field_w,
            h: self.stair_y,
        }
    }

    fn right_side_top(&self) -> Rect {
        Rect {
            x: self.width - self.side_field_w,
            y: 0,
            w: self.side_field_w,
            h: self.stair_y,
        }
    }

    fn top_bar_row(&self) -> Rect {
        Rect {
            x: self.central_x(),
            y: 0,
            w: self.central_w(),
            h: self.top_h,
        }
    }

    fn main_bar_row(&self) -> Rect {
        Rect {
            x: self.central_x(),
            y: self.top_h,
            w: self.central_w(),
            h: self.main_h,
        }
    }

    fn stair_row(&self) -> Rect {
        Rect {
            x: self.central_x(),
            y: self.stair_y,
            w: self.central_w(),
            h: self.row_h,
        }
    }

    fn left_stair_cap(&self) -> Rect {
        Rect {
            x: 0,
            y: self.stair_y,
            w: self.side_field_w,
            h: self.row_h,
        }
    }

    fn right_stair_cap(&self) -> Rect {
        Rect {
            x: self.width - self.side_field_w,
            y: self.stair_y,
            w: self.side_field_w,
            h: self.row_h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pq_known_points() {
        assert!((pq_oetf(0.0) - 0.000000730955).abs() < 0.000000001);
        assert!((pq_oetf(100.0) - 0.508078).abs() < 0.000001);
        assert!((pq_oetf(10000.0) - 1.0).abs() < 0.000001);
    }

    #[test]
    fn hlg_known_points() {
        assert!((hlg_oetf(0.0) - 0.0).abs() < 0.000001);
        assert!((hlg_oetf(1.0 / 12.0) - 0.5).abs() < 0.000001);
        assert!((hlg_oetf(1.0) - 1.0).abs() < 0.000001);
    }

    #[test]
    fn rgb_to_ycbcr_known_colors() {
        assert_eq!(
            ycbcr_from_rgb(grey(LIMITED_BLACK)),
            Ycbcr {
                y: LIMITED_BLACK,
                cb: CHROMA_NEUTRAL,
                cr: CHROMA_NEUTRAL
            }
        );
        assert_eq!(
            ycbcr_from_rgb(grey(LIMITED_WHITE)),
            Ycbcr {
                y: LIMITED_WHITE,
                cb: CHROMA_NEUTRAL,
                cr: CHROMA_NEUTRAL
            }
        );
        assert_eq!(
            ycbcr_from_rgb(RgbCode {
                r: LIMITED_WHITE,
                g: LIMITED_BLACK,
                b: LIMITED_BLACK
            }),
            Ycbcr {
                y: 294,
                cb: 387,
                cr: 960
            }
        );
    }

    #[test]
    fn representative_hlg_regions() {
        let frame = generate_frame(GeneratorOptions {
            transfer: Transfer::Hlg,
            width: BASE_WIDTH,
            height: BASE_HEIGHT,
            frames: 1,
        })
        .unwrap();

        assert_eq!(frame.pixel(10, 10), ycbcr_from_rgb(grey(GREY_40_CODE)));
        assert_eq!(frame.pixel(250, 10), ycbcr_from_rgb(grey(LIMITED_WHITE)));
        assert_eq!(
            frame.pixel(450, 100),
            ycbcr_from_rgb(RgbCode {
                r: HLG_MAIN_BAR_CODE,
                g: HLG_MAIN_BAR_CODE,
                b: LIMITED_BLACK
            })
        );
        assert_eq!(frame.pixel(250, 650), ycbcr_from_rgb(grey(RAMP_MIN_CODE)));
        assert_eq!(frame.pixel(250, 730).cb, CHROMA_NEUTRAL);
        assert_eq!(frame.pixel(250, 730).cr, CHROMA_NEUTRAL);
        assert_eq!(
            frame.pixel(10, 830),
            Ycbcr {
                y: 694,
                cb: 307,
                cr: 526
            }
        );
        assert_eq!(frame.pixel(400, 830), ycbcr_from_rgb(grey(46)));
        assert_eq!(
            frame.pixel(1000, 830),
            ycbcr_from_rgb(grey(HLG_MAIN_BAR_CODE))
        );
        assert_eq!(
            frame.pixel(1700, 830),
            Ycbcr {
                y: 406,
                cb: 674,
                cr: 681
            }
        );
    }

    #[test]
    fn hlg_stair_and_ramp_boundaries() {
        let frame = generate_frame(GeneratorOptions {
            transfer: Transfer::Hlg,
            width: BASE_WIDTH,
            height: BASE_HEIGHT,
            frames: 1,
        })
        .unwrap();

        assert_eq!(frame.pixel(1164, 650).y, 590);
        assert_eq!(frame.pixel(1165, 650).y, 677);
        assert_eq!(frame.pixel(239, 730).y, LIMITED_BLACK);
        assert_eq!(frame.pixel(240, 730).y, RAMP_MIN_CODE);
        assert_eq!(frame.pixel(699, 730).y, RAMP_MIN_CODE);
        assert_eq!(frame.pixel(700, 730).y, 5);
        assert_eq!(frame.pixel(1714, 730).y, RAMP_MAX_CODE);
        assert_eq!(frame.pixel(1715, 730).y, RAMP_MAX_CODE);
    }

    #[test]
    fn accepts_native_4k_and_8k_scales() {
        for (width, height) in [
            (BASE_WIDTH, BASE_HEIGHT),
            (UHD_WIDTH, UHD_HEIGHT),
            (EIGHT_K_WIDTH, EIGHT_K_HEIGHT),
        ] {
            assert!(validate_options(GeneratorOptions {
                transfer: Transfer::Pq,
                width,
                height,
                frames: 1,
            })
            .is_ok());
        }
    }

    #[test]
    fn representative_hlg_regions_scale_to_4k() {
        let frame = generate_frame(GeneratorOptions {
            transfer: Transfer::Hlg,
            width: UHD_WIDTH,
            height: UHD_HEIGHT,
            frames: 1,
        })
        .unwrap();

        assert_eq!(frame.pixel(20, 20), ycbcr_from_rgb(grey(GREY_40_CODE)));
        assert_eq!(frame.pixel(500, 20), ycbcr_from_rgb(grey(LIMITED_WHITE)));
        assert_eq!(frame.pixel(500, 1300), ycbcr_from_rgb(grey(RAMP_MIN_CODE)));
        assert_eq!(
            frame.pixel(2000, 1660),
            ycbcr_from_rgb(grey(HLG_MAIN_BAR_CODE))
        );
    }

    #[test]
    fn rejects_non_integer_scale() {
        assert!(generate_frame(GeneratorOptions {
            transfer: Transfer::Pq,
            width: TEST_NON_INTEGER_SCALE_WIDTH,
            height: TEST_NON_INTEGER_SCALE_HEIGHT,
            frames: 1,
        })
        .is_err());
    }
}
