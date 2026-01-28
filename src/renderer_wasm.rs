use crate::types::{Image, Polygon};

/// Raw RGBA pixel buffer for WASM rendering
pub struct PixelBuffer {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height * 4],
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    #[inline]
    fn set_pixel_blended(&mut self, x: usize, y: usize, color: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = (y * self.width + x) * 4;
        let alpha = color[3] as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        self.data[idx] = (color[0] as f32 * alpha + self.data[idx] as f32 * inv_alpha) as u8;
        self.data[idx + 1] =
            (color[1] as f32 * alpha + self.data[idx + 1] as f32 * inv_alpha) as u8;
        self.data[idx + 2] =
            (color[2] as f32 * alpha + self.data[idx + 2] as f32 * inv_alpha) as u8;
        self.data[idx + 3] = 255; // Fully opaque background
    }
}

/// Edge structure for scanline algorithm
#[derive(Clone)]
struct Edge {
    y_max: i32,
    x_current: f32,
    inv_slope: f32,
}

/// Scanline polygon fill algorithm
fn fill_polygon(buffer: &mut PixelBuffer, polygon: &Polygon) {
    if polygon.points.len() < 3 {
        return;
    }

    let points: Vec<(i32, i32)> = polygon
        .points
        .iter()
        .map(|(x, y)| (*x as i32, *y as i32))
        .collect();

    // Find y bounds
    let y_min = points.iter().map(|(_, y)| *y).min().unwrap_or(0).max(0);
    let y_max = points
        .iter()
        .map(|(_, y)| *y)
        .max()
        .unwrap_or(0)
        .min(buffer.height as i32 - 1);

    if y_min > y_max {
        return;
    }

    // Build edge table
    let n = points.len();
    let mut edge_table: Vec<Vec<Edge>> = vec![Vec::new(); (y_max - y_min + 1) as usize];

    for i in 0..n {
        let (x0, y0) = points[i];
        let (x1, y1) = points[(i + 1) % n];

        if y0 == y1 {
            continue; // Skip horizontal edges
        }

        let (x_lower, y_lower, x_upper, y_upper) = if y0 < y1 {
            (x0 as f32, y0, x1 as f32, y1)
        } else {
            (x1 as f32, y1, x0 as f32, y0)
        };

        if y_lower >= buffer.height as i32 || y_upper < 0 {
            continue;
        }

        let inv_slope = (x_upper - x_lower) / (y_upper - y_lower) as f32;

        let bucket_idx = (y_lower.max(0) - y_min) as usize;
        if bucket_idx < edge_table.len() {
            edge_table[bucket_idx].push(Edge {
                y_max: y_upper,
                x_current: x_lower,
                inv_slope,
            });
        }
    }

    // Scanline fill
    let mut active_edges: Vec<Edge> = Vec::new();

    for y in y_min..=y_max {
        if y < 0 || y >= buffer.height as i32 {
            continue;
        }

        // Add new edges from edge table
        let bucket_idx = (y - y_min) as usize;
        if bucket_idx < edge_table.len() {
            active_edges.extend(edge_table[bucket_idx].drain(..));
        }

        // Remove edges that end at this scanline
        active_edges.retain(|e| e.y_max > y);

        // Sort active edges by x
        active_edges.sort_by(|a, b| a.x_current.partial_cmp(&b.x_current).unwrap());

        // Fill between pairs of edges
        let mut i = 0;
        while i + 1 < active_edges.len() {
            let x_start = (active_edges[i].x_current.ceil() as i32).max(0);
            let x_end = (active_edges[i + 1].x_current.floor() as i32).min(buffer.width as i32 - 1);

            for x in x_start..=x_end {
                buffer.set_pixel_blended(x as usize, y as usize, polygon.colour);
            }

            i += 2;
        }

        // Update x positions for next scanline
        for edge in active_edges.iter_mut() {
            edge.x_current += edge.inv_slope;
        }
    }
}

/// Render an Image to a PixelBuffer using scanline fill
pub fn render_image(img: &Image) -> PixelBuffer {
    let mut buffer = PixelBuffer::new(img.width, img.height);

    for polygon in &img.polygon {
        fill_polygon(&mut buffer, polygon);
    }

    buffer
}

/// Render into an existing buffer (reuse allocation)
pub fn render_image_into(img: &Image, buffer: &mut PixelBuffer) {
    buffer.clear();
    for polygon in &img.polygon {
        fill_polygon(buffer, polygon);
    }
}
