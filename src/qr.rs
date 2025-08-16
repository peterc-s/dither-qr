use anyhow::{Context, Result};
use qrcode::{EcLevel, QrCode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Data,
    Free,
    Locked,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub is_black: bool,
    pub cell_type: CellType,
}

pub fn generate_qr_data(text: &str, ec_level: EcLevel) -> Result<Vec<Vec<bool>>> {
    let code = QrCode::with_error_correction_level(text, ec_level)
        .context("Failed to generate QR code")?;

    let modules = code.to_colors();
    let width = code.width();

    let mut qr_data = vec![vec![false; width]; width];
    for y in 0..width {
        for x in 0..width {
            qr_data[y][x] = matches!(modules[y * width + x], qrcode::Color::Dark);
        }
    }

    Ok(qr_data)
}
