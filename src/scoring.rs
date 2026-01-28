#[cfg(feature = "cli")]
use image::RgbaImage;

#[cfg(feature = "cli")]
use crate::renderer::render_image;
#[cfg(feature = "cli")]
use crate::types::Image;

#[cfg(feature = "cli")]
pub type CompareFn = fn(&RgbaImage, &RgbaImage) -> f32;

#[cfg(feature = "cli")]
pub fn sad_compare(target: &RgbaImage, rendered: &RgbaImage) -> f32 {
    let total_diff: u64 = target
        .as_raw()
        .iter()
        .zip(rendered.as_raw().iter())
        .map(|(a, b)| (*a as i16 - *b as i16).abs() as u64)
        .sum();

    let max_diff = target.as_raw().len() as u64 * 255;
    1.0 - (total_diff as f32 / max_diff as f32)
}

#[cfg(feature = "cli")]
pub fn mse_compare(target: &RgbaImage, rendered: &RgbaImage) -> f32 {
    let total_sq_diff: u64 = target
        .as_raw()
        .iter()
        .zip(rendered.as_raw().iter())
        .map(|(a, b)| {
            let diff = *a as i32 - *b as i32;
            (diff * diff) as u64
        })
        .sum();

    let max_sq_diff = target.as_raw().len() as u64 * 255 * 255;
    1.0 - (total_sq_diff as f32 / max_sq_diff as f32)
}

#[cfg(feature = "cli")]
pub fn score_images(
    images: Vec<Image>,
    target: &RgbaImage,
    compare_fn: CompareFn,
) -> Vec<(f32, Image)> {
    images
        .into_iter()
        .map(|img| {
            let rendered = render_image(&img);
            let score = compare_fn(target, &rendered);
            (score, img)
        })
        .collect()
}

// WASM-compatible raw pixel comparison functions

/// Compare two raw RGBA pixel arrays using Sum of Absolute Differences
/// Returns accuracy as a float from 0.0 to 1.0
pub fn sad_compare_raw(target: &[u8], rendered: &[u8]) -> f32 {
    if target.len() != rendered.len() || target.is_empty() {
        return 0.0;
    }

    let total_diff: u64 = target
        .iter()
        .zip(rendered.iter())
        .map(|(a, b)| (*a as i16 - *b as i16).abs() as u64)
        .sum();

    let max_diff = target.len() as u64 * 255;
    1.0 - (total_diff as f32 / max_diff as f32)
}

/// Compare two raw RGBA pixel arrays using Mean Squared Error
/// Returns accuracy as a float from 0.0 to 1.0
pub fn mse_compare_raw(target: &[u8], rendered: &[u8]) -> f32 {
    if target.len() != rendered.len() || target.is_empty() {
        return 0.0;
    }

    let total_sq_diff: u64 = target
        .iter()
        .zip(rendered.iter())
        .map(|(a, b)| {
            let diff = *a as i32 - *b as i32;
            (diff * diff) as u64
        })
        .sum();

    let max_sq_diff = target.len() as u64 * 255 * 255;
    1.0 - (total_sq_diff as f32 / max_sq_diff as f32)
}
