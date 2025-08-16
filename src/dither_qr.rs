use crate::qr::{Cell, CellType};
use anyhow::{Context, Result};
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use ndarray::Array2;
use rayon::prelude::*;

pub struct DitheredQR {
    big_size: usize,
    cells: Array2<Cell>,
    targets: Array2<f32>,
    gamma: f32,
    contrast: f32,
    brightness: f32,
}

impl DitheredQR {
    pub fn new(
        qr_data: &[Vec<bool>],
        ratio: usize,
        gamma: f32,
        contrast: f32,
        brightness: f32,
    ) -> Result<Self> {
        let qr_size = qr_data.len();
        let big_size = qr_size * ratio;
        let center_offset = ratio / 2;

        // Build every big-grid cell independently in parallel, in row-major order.
        let flat_cells: Vec<Cell> = (0..big_size * big_size)
            .into_par_iter()
            .map(|i| {
                let y = i / big_size;
                let x = i % big_size;

                // Map big grid -> QR module + subcell
                let qr_y = y / ratio;
                let qr_x = x / ratio;
                let sub_y = y % ratio;
                let sub_x = x % ratio;

                let is_black = qr_data[qr_y][qr_x];
                let is_locked = Self::is_locked_position(qr_x, qr_y, qr_size);

                let cell_type = if is_locked {
                    CellType::Locked
                } else if sub_x == center_offset && sub_y == center_offset {
                    CellType::Data
                } else {
                    CellType::Free
                };

                Cell {
                    is_black,
                    cell_type,
                }
            })
            .collect();

        let cells = ndarray::Array2::from_shape_vec((big_size, big_size), flat_cells)
            .context("Failed to construct ndarray for QR cells")?;
        let targets = ndarray::Array2::zeros((big_size, big_size));

        Ok(Self {
            big_size,
            cells,
            targets,
            gamma,
            contrast,
            brightness,
        })
    }

    fn is_locked_position(x: usize, y: usize, size: usize) -> bool {
        // Timing patterns
        if x == 6 || y == 6 {
            return true;
        }

        // Finder patterns (top-left)
        if x < 8 && y < 8 {
            return true;
        }

        // Finder pattern (top-right)
        if x > size - 9 && y < 8 {
            return true;
        }

        // Finder pattern (bottom-left)
        if y > size - 9 && x < 8 {
            return true;
        }

        // Alignment pattern (bottom-right)
        if size >= 25 && x > size - 10 && y > size - 10 && x < size - 4 && y < size - 4 {
            return true;
        }

        false
    }

    pub fn process_image(&mut self, img: &RgbImage) -> Result<()> {
        let resized = image::imageops::resize(
            img,
            self.big_size as u32,
            self.big_size as u32,
            image::imageops::FilterType::Lanczos3,
        );

        let big_size = self.big_size;

        let flat_targets: Vec<f32> = (0..big_size * big_size)
            .into_par_iter()
            .map(|i| {
                let y = i / big_size;
                let x = i % big_size;

                let p = resized.get_pixel(x as u32, y as u32);
                let gray = p.to_luma()[0] as f32 / 255.0;

                let gamma_corrected = gray.powf(self.gamma);
                let adjusted = gamma_corrected * self.contrast + self.brightness;
                adjusted.clamp(0.0, 1.0)
            })
            .collect();

        self.targets = ndarray::Array2::from_shape_vec((big_size, big_size), flat_targets)
            .context("Failed to construct ndarray from image.")?;

        Ok(())
    }

    pub fn apply_dithering(&mut self) {
        // First pass: process data cells (center cells of QR modules) with symmetric error diffusion
        for y in 0..self.big_size {
            for x in 0..self.big_size {
                let cell = self.cells[[y, x]];
                if cell.cell_type != CellType::Data {
                    continue;
                }

                let target = self.targets[[y, x]];
                let actual = if cell.is_black { 0.0 } else { 1.0 };
                let error = actual - target;

                // Symmetric 8-neighbor error diffusion for data cells
                self.bump_target(x as i32 + 1, y as i32, error * 3.0 / 16.0);
                self.bump_target(x as i32 - 1, y as i32, error * 3.0 / 16.0);
                self.bump_target(x as i32, y as i32 + 1, error * 3.0 / 16.0);
                self.bump_target(x as i32, y as i32 - 1, error * 3.0 / 16.0);
                self.bump_target(x as i32 + 1, y as i32 + 1, error * 1.0 / 16.0);
                self.bump_target(x as i32 - 1, y as i32 + 1, error * 1.0 / 16.0);
                self.bump_target(x as i32 + 1, y as i32 - 1, error * 1.0 / 16.0);
                self.bump_target(x as i32 - 1, y as i32 - 1, error * 1.0 / 16.0);
            }
        }

        // Second pass: process free cells (non-center cells in unlocked areas) with Floyd-Steinberg
        for y in 0..self.big_size {
            for x in 0..self.big_size {
                let mut cell = self.cells[[y, x]];

                if cell.cell_type != CellType::Free {
                    continue;
                }

                let target = self.targets[[y, x]];

                // Decide black or white based on threshold
                let new_is_black = target < 0.5;
                cell.is_black = new_is_black;
                self.cells[[y, x]] = cell;

                let actual = if new_is_black { 0.0 } else { 1.0 };
                let error = actual - target;

                // Floyd-Steinberg error diffusion with dynamic weighting
                let a = self.is_free(x as i32 + 1, y as i32);
                let b = self.is_free(x as i32 - 1, y as i32 + 1);
                let c = self.is_free(x as i32, y as i32 + 1);
                let d = self.is_free(x as i32 + 1, y as i32 + 1);

                let total = (if a { 7.0 } else { 0.0 })
                    + (if b { 3.0 } else { 0.0 })
                    + (if c { 5.0 } else { 0.0 })
                    + (if d { 1.0 } else { 0.0 });

                if total > 0.0 {
                    if a {
                        self.bump_target(x as i32 + 1, y as i32, error * 7.0 / total);
                    }
                    if b {
                        self.bump_target(x as i32 - 1, y as i32 + 1, error * 3.0 / total);
                    }
                    if c {
                        self.bump_target(x as i32, y as i32 + 1, error * 5.0 / total);
                    }
                    if d {
                        self.bump_target(x as i32 + 1, y as i32 + 1, error * 1.0 / total);
                    }
                }
            }
        }
    }

    fn bump_target(&mut self, x: i32, y: i32, error: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.big_size && (y as usize) < self.big_size {
            self.targets[[y as usize, x as usize]] -= error;
            self.targets[[y as usize, x as usize]] =
                self.targets[[y as usize, x as usize]].clamp(0.0, 1.0);
        }
    }

    fn is_free(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || (x as usize) >= self.big_size || (y as usize) >= self.big_size {
            return false;
        }
        self.cells[[y as usize, x as usize]].cell_type == CellType::Free
    }

    pub fn render_to_image(&self) -> RgbImage {
        ImageBuffer::from_fn(self.big_size as u32, self.big_size as u32, |x, y| {
            let cell = self.cells[[y as usize, x as usize]];
            if cell.is_black {
                Rgb([0, 0, 0])
            } else {
                Rgb([255, 255, 255])
            }
        })
    }
}
