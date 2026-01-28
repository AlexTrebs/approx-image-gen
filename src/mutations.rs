use rand::{prelude::*, rng};

use crate::generations::{generate_random_point, generate_random_polygon};
use crate::types::Image;

const POINT_MOVE_DELTA: f32 = 5.0;
const POLYGON_MOVE_DELTA: f32 = 3.0;
const COLOUR_DELTA: i16 = 20;
const MIN_POLYGONS: usize = 10;

const MUTATION_TYPE_CHANCE: &[(&str, f32)] = &[
  ("move-point", 0.30),
  ("change-colour", 0.30),
  ("move-polygon", 0.15),
  ("reorder-polygon", 0.10),
  ("add-polygon", 0.05),
  ("remove-polygon", 0.05),
  ("new-point", 0.03),
  ("delete-point", 0.02),
];

pub fn add_point(mut image: Image) -> Image {
  if image.polygon.is_empty() {
    return image;
  }
  let mutate_poly = rng().random_range(0..image.polygon.len());
  let point = generate_random_point(image.width, image.height);
  image.polygon[mutate_poly].points.push(point);
  image
}

pub fn delete_point(mut image: Image) -> Image {
  if image.polygon.is_empty() {
    return image;
  }
  let mutate_poly = rng().random_range(0..image.polygon.len());
  if image.polygon[mutate_poly].points.len() > 3 {
    let point = rng().random_range(0..image.polygon[mutate_poly].points.len());
    image.polygon[mutate_poly].points.remove(point);
  }
  image
}

pub fn move_point(mut image: Image) -> Image {
  if image.polygon.is_empty() {
    return image;
  }
  let mutate_poly = rng().random_range(0..image.polygon.len());
  if image.polygon[mutate_poly].points.is_empty() {
    return image;
  }
  let point_idx = rng().random_range(0..image.polygon[mutate_poly].points.len());
  let point = &mut image.polygon[mutate_poly].points[point_idx];

  let dx: f32 = rng().random_range(-POINT_MOVE_DELTA..=POINT_MOVE_DELTA);
  let dy: f32 = rng().random_range(-POINT_MOVE_DELTA..=POINT_MOVE_DELTA);

  point.0 = (point.0 + dx).clamp(0.0, image.width as f32 - 1.0);
  point.1 = (point.1 + dy).clamp(0.0, image.height as f32 - 1.0);
  image
}

pub fn move_polygon(mut image: Image) -> Image {
  if image.polygon.is_empty() {
    return image;
  }
  let mutate_poly = rng().random_range(0..image.polygon.len());

  let dx: f32 = rng().random_range(-POLYGON_MOVE_DELTA..=POLYGON_MOVE_DELTA);
  let dy: f32 = rng().random_range(-POLYGON_MOVE_DELTA..=POLYGON_MOVE_DELTA);

  for point in &mut image.polygon[mutate_poly].points {
    point.0 = (point.0 + dx).clamp(0.0, image.width as f32 - 1.0);
    point.1 = (point.1 + dy).clamp(0.0, image.height as f32 - 1.0);
  }
  image
}

pub fn reorder_polygon(mut image: Image) -> Image {
  if image.polygon.len() < 2 {
    return image;
  }
  let idx_a = rng().random_range(0..image.polygon.len());
  let idx_b = rng().random_range(0..image.polygon.len());
  image.polygon.swap(idx_a, idx_b);
  image
}

pub fn change_colour(mut image: Image) -> Image {
  if image.polygon.is_empty() {
    return image;
  }
  let mutate_poly = rng().random_range(0..image.polygon.len());
  let colour = &mut image.polygon[mutate_poly].colour;

  let channel: usize = rng().random_range(0..4);
  let delta: i16 = rng().random_range(-COLOUR_DELTA..=COLOUR_DELTA);

  colour[channel] = (colour[channel] as i16 + delta).clamp(0, 255) as u8;
  image
}

pub fn add_polygon(mut image: Image) -> Image {
  let new_poly = generate_random_polygon(image.width, image.height);
  image.polygon.push(new_poly);
  image
}

pub fn remove_polygon(mut image: Image) -> Image {
  if image.polygon.len() > MIN_POLYGONS {
    let idx = rng().random_range(0..image.polygon.len());
    image.polygon.remove(idx);
  }
  image
}

pub fn mutate_image(image: Image) -> Image {
  let mut rand_prob: f32 = rng().random_range(0.0..1.0);
  let mut mutation_type: &str = "move-point"; // default

  for (mut_type, prob) in MUTATION_TYPE_CHANCE.iter() {
    if rand_prob <= *prob {
      mutation_type = mut_type;
      break;
    } else {
      rand_prob -= *prob;
    }
  }

  match mutation_type {
    "move-point" => move_point(image),
    "change-colour" => change_colour(image),
    "move-polygon" => move_polygon(image),
    "reorder-polygon" => reorder_polygon(image),
    "add-polygon" => add_polygon(image),
    "remove-polygon" => remove_polygon(image),
    "new-point" => add_point(image),
    "delete-point" => delete_point(image),
    _ => move_point(image),
  }
}
