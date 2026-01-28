#[derive(Clone)]
pub struct Polygon {
  pub points: Vec<(f32, f32)>,
  pub colour: [u8; 4],
}

#[derive(Clone)]
pub struct Image {
  pub polygon: Vec<Polygon>,
  pub width: usize,
  pub height: usize,
}
