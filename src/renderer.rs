use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_polygon_mut;
use imageproc::point::Point;

use crate::types::Image;

pub fn render_image(img: &Image) -> RgbaImage {
  let mut canvas = RgbaImage::new(img.width as u32, img.height as u32);

  for poly in &img.polygon {
    let points: Vec<Point<i32>> = poly
      .points
      .iter()
      .map(|(x, y)| Point::new(*x as i32, *y as i32))
      .collect();

    let colour = Rgba(poly.colour);
    draw_polygon_mut(&mut canvas, &points, colour);
  }

  canvas
}
