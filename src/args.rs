use clap::{Parser, ValueEnum};
use qrcode::EcLevel;
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "UPPER")]
pub enum EcArg {
    L,
    M,
    Q,
    H,
}

impl From<EcArg> for EcLevel {
    fn from(v: EcArg) -> Self {
        match v {
            EcArg::L => EcLevel::L,
            EcArg::M => EcLevel::M,
            EcArg::Q => EcLevel::Q,
            EcArg::H => EcLevel::H,
        }
    }
}

#[derive(Parser)]
#[command(name = "dithered-qr")]
#[command(about = "Generate dithered QR codes with image overlay using Floyd-Steinberg dithering")]
pub struct Args {
    /// Text to encode in the QR code
    #[arg(short, long)]
    pub text: String,

    /// Input image path
    #[arg(short, long)]
    pub image: PathBuf,

    /// Output image path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Cell subdivision ratio (default: 3)
    #[arg(short, long, default_value = "3")]
    pub ratio: usize,

    /// Gamma correction (default: 2.2)
    #[arg(short, long, default_value = "2.2")]
    pub gamma: f32,

    /// Contrast adjustment (default: 1.0)
    #[arg(short = 'c', long, default_value = "1.0")]
    pub contrast: f32,

    /// Brightness adjustment (default: 0.0)
    #[arg(short, long, default_value = "0.0")]
    pub brightness: f32,

    /// QR code error correction level (L, M, Q, H)
    #[arg(short = 'e', long, default_value = "L")]
    pub error_correction: EcArg,

    /// Upscale factor for output image (default: 1, no upscaling)
    #[arg(short = 'u', long, default_value = "1")]
    pub upscale: u32,
}
