use clap::Parser;
use qrcode::QrCode;
use qrcode::Color;
use ravif::{Encoder, Img, RGBA8};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// Generate a QR code image from text (webp by default).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Message to encode into a QR code. If omitted, reads from stdin.
    #[arg(short = 'm', long = "message")]
    message: Option<String>,

    /// Output file name (default: qr.<ext>)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Output format: webp (default), svg, or avif.
    #[arg(
        short = 'f',
        long = "format",
        default_value = "webp",
        value_enum
    )]
    format: Format,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Format {
    Webp,
    Svg,
    Avif,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Format::Webp => "webp",
            Format::Svg => "svg",
            Format::Avif => "avif",
        }
    }
}

fn main() {
    let args = Args::parse();

    let text = match args.message {
        Some(m) => m,
        None => {
            let mut buf = String::new();
            eprint!("Enter text (Ctrl+D to create, Ctrl+C to cancel): ");
            io::stderr().flush().unwrap();
            io::stdin()
                .read_to_string(&mut buf)
                .expect("failed to read stdin");
            buf.trim_end().to_string()
        }
    };

    let qr = QrCode::new(&text).expect("failed to generate QR code");
    let output = args
        .output
        .unwrap_or_else(|| format!("{}.{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), args.format.extension()));

    let bytes: Vec<u8> = match args.format {
        Format::Webp => {
            let width = qr.width() as u32;
            let mut pixels: Vec<u8> = Vec::with_capacity((width * width * 4) as usize);
            for y in 0..width {
                for x in 0..width {
                    let color = qr[(x as usize, y as usize)];
                    let (r, g, b, a) = match color {
                        Color::Dark => (0, 0, 0, 255),
                        Color::Light => (255, 255, 255, 255),
                    };
                    pixels.extend_from_slice(&[r, g, b, a]);
                }
            }
            let mut buf = Vec::new();
            let encoder = image_webp::WebPEncoder::new(&mut buf);
            encoder
                .encode(&pixels, width, width, image_webp::ColorType::Rgba8)
                .expect("failed to encode WebP");
            buf
        }
        Format::Svg => {
            qr.render::<qrcode::render::svg::Color>()
                .build()
                .into_bytes()
        }
        Format::Avif => {
            let width = qr.width() as u32;
            let mut pixels: Vec<RGBA8> = Vec::with_capacity((width * width) as usize);
            for y in 0..width {
                for x in 0..width {
                    let color = qr[(x as usize, y as usize)];
                    let (r, g, b, a) = match color {
                        Color::Dark => (0, 0, 0, 255),
                        Color::Light => (255, 255, 255, 255),
                    };
                    pixels.push(RGBA8::new(r, g, b, a));
                }
            }
            let encoder = Encoder::new();
            let img = Img::new(&pixels[..], width as usize, width as usize);
            let encoded = encoder.encode_rgba(img).expect("failed to encode AVIF");
            encoded.avif_file
        }
    };

    let path = Path::new(&output);
    let mut f = File::create(path).expect(&format!("failed to create {}", output));
    f.write_all(&bytes)
        .expect(&format!("failed to write {}", output));

    println!("{}", output);
}
