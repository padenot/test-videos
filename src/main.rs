use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;

use bt2111_gen::{write_y4m, GeneratorOptions, Transfer};

fn main() {
    if let Err(err) = run() {
        eprintln!("bt2111-gen: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    let file = File::create(&args.output)
        .map_err(|err| format!("failed to create {}: {err}", args.output.display()))?;
    let mut writer = BufWriter::new(file);
    write_y4m(&mut writer, args.options).map_err(|err| format!("failed to write Y4M: {err}"))
}

struct Args {
    options: GeneratorOptions,
    output: PathBuf,
}

fn parse_args<I>(mut args: I) -> Result<Args, String>
where
    I: Iterator<Item = String>,
{
    let mut transfer = None;
    let mut resolution = None;
    let mut frames = None;
    let mut output = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--transfer" => {
                let value = next_value(&mut args, "--transfer")?;
                transfer = Some(parse_transfer(&value)?);
            }
            "--resolution" => {
                let value = next_value(&mut args, "--resolution")?;
                resolution = Some(parse_resolution(&value)?);
            }
            "--frames" => {
                let value = next_value(&mut args, "--frames")?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --frames value: {value}"))?;
                if parsed == 0 {
                    return Err("--frames must be greater than zero".to_string());
                }
                frames = Some(parsed);
            }
            "--output" => {
                output = Some(PathBuf::from(next_value(&mut args, "--output")?));
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let (width, height) = resolution.unwrap_or((1920, 1080));
    Ok(Args {
        options: GeneratorOptions {
            transfer: transfer.ok_or("missing --transfer pq|hlg")?,
            width,
            height,
            frames: frames.unwrap_or(1),
        },
        output: output.ok_or("missing --output out.y4m")?,
    })
}

fn next_value<I>(args: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value after {name}"))
}

fn parse_resolution(value: &str) -> Result<(usize, usize), String> {
    match value {
        "fhd" | "1080p" | "2k" | "1920x1080" => return Ok((1920, 1080)),
        "uhd" | "4k" | "3840x2160" => return Ok((3840, 2160)),
        "8k" | "7680x4320" => return Ok((7680, 4320)),
        _ => {}
    }

    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| format!("invalid --resolution value: {value}"))?;
    let width = width
        .parse::<usize>()
        .map_err(|_| format!("invalid width in --resolution: {value}"))?;
    let height = height
        .parse::<usize>()
        .map_err(|_| format!("invalid height in --resolution: {value}"))?;
    if width == 0 || height == 0 {
        return Err("--resolution dimensions must be greater than zero".to_string());
    }
    Ok((width, height))
}

fn print_help() {
    println!(
        "Usage: bt2111-gen --transfer pq|hlg [--resolution fhd|4k|8k|WxH] [--frames N] --output out.y4m"
    );
}

fn parse_transfer(value: &str) -> Result<Transfer, String> {
    match value {
        "pq" => Ok(Transfer::Pq),
        "hlg" => Ok(Transfer::Hlg),
        _ => Err(format!("invalid --transfer value: {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_resolution_aliases() {
        assert_eq!(parse_resolution("fhd").unwrap(), (1920, 1080));
        assert_eq!(parse_resolution("4k").unwrap(), (3840, 2160));
        assert_eq!(parse_resolution("8k").unwrap(), (7680, 4320));
    }

    #[test]
    fn parses_explicit_resolution() {
        assert_eq!(parse_resolution("5760x3240").unwrap(), (5760, 3240));
    }
}
