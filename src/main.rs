use clap::Parser;
use colored::Colorize;
use image::{DynamicImage, GenericImageView, Pixel};
use std::path::PathBuf;

const ASCII_DENSE: &[char] = &[
    '$', '@', 'B', '%', '8', '&', 'W', 'M', '#', '*', 'o', 'a', 'h', 'k', 'b', 'd', 'p', 'q',
    'w', 'm', 'Z', 'O', '0', 'Q', 'L', 'C', 'J', 'U', 'Y', 'X', 'z', 'c', 'v', 'u', 'n', 'x',
    'r', 'j', 'f', 't', '/', '\\', '|', '(', ')', '1', '{', '}', '[', ']', '?', '-', '_', '+',
    '~', '<', '>', 'i', '!', 'l', 'I', ';', ':', ',', '"', '^', '`', '\'', '.', ' ',
];

const ASCII_SIMPLE: &[char] = &[
    '@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.', ' ',
];

const ASCII_BLOCKS: &[char] = &['█', '▓', '▒', '░', ' '];

#[derive(clap::ValueEnum, Clone, Debug)]
enum CharSet {
    Dense,
    Simple,
    Blocks,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ColorMode {
    None,
    Color,
    Gray,
}

#[derive(Parser, Debug)]
#[command(name = "img2ascii")]
#[command(about = "Transform images into ASCII art", long_about = None)]
struct Args {
    #[arg(value_name = "IMAGE")]
    input: PathBuf,

    #[arg(short, long, default_value_t = 100)]
    width: u32,

    #[arg(long)]
    height: Option<u32>,

    #[arg(short, long, value_enum, default_value_t = CharSet::Dense)]
    charset: CharSet,

    #[arg(short = 'C', long, value_enum, default_value_t = ColorMode::None)]
    color: ColorMode,

    #[arg(short, long)]
    invert: bool,

    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn luminance(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

fn pixel_to_char(lum: f32, charset: &[char], invert: bool) -> char {
    let normalized = if invert { 1.0 - lum / 255.0 } else { lum / 255.0 };
    let idx = ((1.0 - normalized) * (charset.len() - 1) as f32).round() as usize;
    charset[idx.min(charset.len() - 1)]
}

fn image_to_ascii(img: &DynamicImage, args: &Args) -> Vec<Vec<(char, u8, u8, u8)>> {
    let charset: &[char] = match args.charset {
        CharSet::Dense => ASCII_DENSE,
        CharSet::Simple => ASCII_SIMPLE,
        CharSet::Blocks => ASCII_BLOCKS,
    };

    let aspect_correction = 0.45_f32;
    let out_w = args.width;
    let out_h = args.height.unwrap_or_else(|| {
        let (w, h) = img.dimensions();
        ((h as f32 / w as f32) * out_w as f32 * aspect_correction).round() as u32
    });

    let resized = img.resize_exact(out_w, out_h, image::imageops::FilterType::Lanczos3);

    let mut rows = Vec::with_capacity(out_h as usize);
    for y in 0..out_h {
        let mut row = Vec::with_capacity(out_w as usize);
        for x in 0..out_w {
            let pixel = resized.get_pixel(x, y);
            let rgba = pixel.to_rgba();
            let (r, g, b, a) = (rgba[0], rgba[1], rgba[2], rgba[3]);

            let (r, g, b) = if a < 128 {
                (255, 255, 255)
            } else {
                (r, g, b)
            };

            let lum = luminance(r, g, b);
            let ch = pixel_to_char(lum, charset, args.invert);
            row.push((ch, r, g, b));
        }
        rows.push(row);
    }

    rows
}

fn render_to_terminal(rows: &[Vec<(char, u8, u8, u8)>], color_mode: &ColorMode) {
    for row in rows {
        for (ch, r, g, b) in row {
            let s = ch.to_string();
            match color_mode {
                ColorMode::None => print!("{s}"),
                ColorMode::Color => print!("{}", s.truecolor(*r, *g, *b)),
                ColorMode::Gray => {
                    let lum = luminance(*r, *g, *b) as u8;
                    print!("{}", s.truecolor(lum, lum, lum));
                }
            }
        }
        println!();
    }
}

fn render_to_string(rows: &[Vec<(char, u8, u8, u8)>]) -> String {
    let mut out = String::new();
    for row in rows {
        for (ch, _, _, _) in row {
            out.push(*ch);
        }
        out.push('\n');
    }
    out
}
// lmao i got so invested i forgot the main func (btw if ur new func is short for function)
fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: file not found — {}", args.input.display());
        std::process::exit(1);
    }

    let img = match image::open(&args.input) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error: could not open image — {e}");
            std::process::exit(1);
        }
    };

    let rows = image_to_ascii(&img, &args);

    if let Some(out_path) = &args.output {
        let text = render_to_string(&rows);
        if let Err(e) = std::fs::write(out_path, text) {
            eprintln!("Error writing file: {e}");
            std::process::exit(1);
        }
        println!("Saved to {}", out_path.display());
    } else {
        render_to_terminal(&rows, &args.color);
    }
}
