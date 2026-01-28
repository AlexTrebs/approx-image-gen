use rand::{prelude::*, rng};

use crate::types::{Image, Polygon};

const MAX_POINTS: usize = 6;
const MIN_POINTS: usize = 3;
const INITIAL_POLYGONS: usize = 50;

pub fn generate_random_colour() -> [u8; 4] {
  [
    rng().random_range(0..=255),
    rng().random_range(0..=255),
    rng().random_range(0..=255),
    rng().random_range(30..=150), // Semi-transparent for layering
  ]
}

pub fn generate_random_point(width: usize, height: usize) -> (f32, f32) {
  (
    rng().random_range(0.0..width as f32),
    rng().random_range(0.0..height as f32),
  )
}

pub fn generate_random_points(width: usize, height: usize) -> Vec<(f32, f32)> {
  let count = rng().random_range(MIN_POINTS..=MAX_POINTS);
  (0..count)
    .map(|_| generate_random_point(width, height))
    .collect()
}

pub fn generate_random_polygon(width: usize, height: usize) -> Polygon {
  Polygon {
    points: generate_random_points(width, height),
    colour: generate_random_colour(),
  }
}

pub fn generate_initial_image(width: usize, height: usize) -> Image {
  Image {
    polygon: (0..INITIAL_POLYGONS)
      .map(|_| generate_random_polygon(width, height))
      .collect(),
    width,
    height,
  }
}
