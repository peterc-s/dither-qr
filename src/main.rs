mod args;
mod dither_qr;
mod qr;

use anyhow::Result;
use args::Args;
use clap::Parser;
use dither_qr::DitheredQR;
use image::imageops;
use qr::generate_qr_data;

fn main() -> Result<()> {
    let args = Args::parse();

    if args.ratio % 2 == 0 {
        return Err(anyhow::anyhow!("Ratio must be odd"));
    }

    println!("Generating QR code for: {}", args.text);
    let qr_data = generate_qr_data(&args.text, args.error_correction.into())?;

    println!("Loading image: {}", args.image.display());
    let img = image::open(&args.image)?.to_rgb8();

    let mut dithered_qr = DitheredQR::new(
        &qr_data,
        args.ratio,
        args.gamma,
        args.contrast,
        args.brightness,
    )?;

    dithered_qr.process_image(&img)?;
    dithered_qr.apply_dithering();

    let mut output_img = dithered_qr.render_to_image();

    if args.upscale > 1 {
        output_img = imageops::resize(
            &output_img,
            output_img.width() * args.upscale,
            output_img.height() * args.upscale,
            imageops::FilterType::Nearest,
        );
    }

    output_img.save(&args.output)?;

    println!("Saved to: {}", args.output.display());
    Ok(())
}
