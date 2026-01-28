use crate::generations::generate_initial_image;
use crate::mutations::mutate_image;
use crate::renderer_wasm::{render_image, PixelBuffer};
use crate::scoring::sad_compare_raw;
use crate::types::Image;
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
pub enum AlgorithmType {
  EvolutionStrategy,
  SimulatedAnnealing,
  DifferentialEvolution,
}

pub struct AlgorithmConfig {
  pub max_iterations: usize,
  pub target_accuracy: f32,
  pub algorithm_type: AlgorithmType,
  // ES specific
  pub es_children_per_parent: usize,
  // SA specific
  pub initial_temp: f32,
  pub cooling_rate: f32,
  // DE specific
  pub population_size: usize,
  pub mutation_factor: f32,
  pub crossover_rate: f32,
}

impl Default for AlgorithmConfig {
  fn default() -> Self {
    Self {
      max_iterations: 100000,
      target_accuracy: 0.95,
      algorithm_type: AlgorithmType::EvolutionStrategy,
      // ES params
      es_children_per_parent: 5,
      // SA params
      initial_temp: 1.0,
      cooling_rate: 0.99995,
      // DE params
      population_size: 6,
      mutation_factor: 0.8,
      crossover_rate: 0.9,
    }
  }
}

pub struct AlgorithmState {
  config: AlgorithmConfig,
  target_pixels: Vec<u8>,
  width: usize,
  height: usize,
  iteration: usize,
  finished: bool,
  // ES state (Evolution Strategy - original algorithm)
  es_parents: Vec<(f32, Image)>,
  es_no_improvement: usize,
  // SA state
  sa_current: Option<Image>,
  sa_current_score: f32,
  sa_best: Option<Image>,
  sa_best_score: f32,
  sa_temperature: f32,
  // DE state
  de_population: Vec<(f32, Image)>,
}

impl AlgorithmState {
  pub fn new(target_pixels: Vec<u8>, width: usize, height: usize, config: AlgorithmConfig) -> Self {
    let mut state = Self {
      target_pixels,
      width,
      height,
      iteration: 0,
      finished: false,
      es_parents: Vec::new(),
      es_no_improvement: 0,
      sa_current: None,
      sa_current_score: 0.0,
      sa_best: None,
      sa_best_score: 0.0,
      sa_temperature: config.initial_temp,
      de_population: Vec::new(),
      config,
    };

    match state.config.algorithm_type {
      AlgorithmType::EvolutionStrategy => state.init_es(),
      AlgorithmType::SimulatedAnnealing => state.init_sa(),
      AlgorithmType::DifferentialEvolution => state.init_de(),
    }

    state
  }

  fn init_es(&mut self) {
    self.es_parents = (0..3)
      .map(|_| {
        let img = generate_initial_image(self.width, self.height);
        let rendered = render_image(&img);
        let score = sad_compare_raw(&self.target_pixels, &rendered.data);
        (score, img)
      })
      .collect();
    self
      .es_parents
      .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
  }

  fn init_sa(&mut self) {
    let img = generate_initial_image(self.width, self.height);
    let rendered = render_image(&img);
    let score = sad_compare_raw(&self.target_pixels, &rendered.data);
    self.sa_current = Some(img.clone());
    self.sa_current_score = score;
    self.sa_best = Some(img);
    self.sa_best_score = score;
    self.sa_temperature = self.config.initial_temp;
  }

  fn init_de(&mut self) {
    self.de_population = (0..self.config.population_size)
      .map(|_| {
        let img = generate_initial_image(self.width, self.height);
        let rendered = render_image(&img);
        let score = sad_compare_raw(&self.target_pixels, &rendered.data);
        (score, img)
      })
      .collect();
    self
      .de_population
      .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
  }

  pub fn step_batch(&mut self, batch_size: usize) -> (bool, PixelBuffer) {
    if self.finished {
      return (true, self.get_best_buffer());
    }

    for _ in 0..batch_size {
      let target_reached = self.get_accuracy() >= self.config.target_accuracy;
      if self.iteration >= self.config.max_iterations || target_reached {
        self.finished = true;
        break;
      }

      match self.config.algorithm_type {
        AlgorithmType::EvolutionStrategy => self.step_es(),
        AlgorithmType::SimulatedAnnealing => self.step_sa(),
        AlgorithmType::DifferentialEvolution => self.step_de(),
      }
      self.iteration += 1;
    }

    (self.finished, self.get_best_buffer())
  }

  fn step_es(&mut self) {
    let old_best = self.es_parents[0].0;

    let mut candidates: Vec<(f32, Image)> =
      Vec::with_capacity(3 * self.config.es_children_per_parent + 3);

    for (_, parent) in &self.es_parents {
      for _ in 0..self.config.es_children_per_parent {
        let child = mutate_image(parent.clone());
        let rendered = render_image(&child);
        let score = sad_compare_raw(&self.target_pixels, &rendered.data);
        candidates.push((score, child));
      }
    }

    candidates.extend(self.es_parents.drain(..));

    // Sort by score (best first)
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let best = candidates.remove(0);
    let second_best = candidates.remove(0);

    // Take worst performer and mutate it heavily (survival of the fittest with a wildcard)
    let (_, mut worst_img) = candidates.pop().unwrap_or_else(|| {
      let img = generate_initial_image(self.width, self.height);
      let rendered = render_image(&img);
      let score = sad_compare_raw(&self.target_pixels, &rendered.data);
      (score, img)
    });

    // Apply random mutations to the worst to give it a fighting chance
    for _ in 0..5 {
      worst_img = mutate_image(worst_img);
    }
    let rendered = render_image(&worst_img);
    let worst_score = sad_compare_raw(&self.target_pixels, &rendered.data);

    self.es_parents = vec![best, second_best, (worst_score, worst_img)];

    // Track improvement
    if self.es_parents[0].0 > old_best {
      self.es_no_improvement = 0;
    } else {
      self.es_no_improvement += 1;
    }

    // If stuck, shake things up more aggressively
    if self.es_no_improvement > 500 {
      let (score, img) = &mut self.es_parents[1];
      for _ in 0..20 {
        *img = mutate_image(img.clone());
      }
      let rendered = render_image(img);
      *score = sad_compare_raw(&self.target_pixels, &rendered.data);
      self.es_no_improvement = 0;
    }
  }

  /// Simulated Annealing step
  fn step_sa(&mut self) {
    let current = self.sa_current.as_ref().unwrap();

    let neighbor = mutate_image(current.clone());
    let rendered = render_image(&neighbor);
    let neighbor_score = sad_compare_raw(&self.target_pixels, &rendered.data);

    // Calculate acceptance probability
    let delta = neighbor_score - self.sa_current_score;
    let accept = if delta > 0.0 {
      true // Always accept improvements
    } else {
      let probability = (delta / self.sa_temperature).exp();
      rand::rng().random::<f32>() < probability
    };

    if accept {
      self.sa_current = Some(neighbor.clone());
      self.sa_current_score = neighbor_score;

      // Track global best
      if neighbor_score > self.sa_best_score {
        self.sa_best = Some(neighbor);
        self.sa_best_score = neighbor_score;
      }
    }

    self.sa_temperature *= self.config.cooling_rate;

    if self.sa_temperature < 0.0001 {
      self.sa_temperature = 0.1;
    }
  }

  /// Differential Evolution step
  fn step_de(&mut self) {
    let pop_size = self.de_population.len();
    let mut new_population: Vec<(f32, Image)> = Vec::with_capacity(pop_size);

    for i in 0..pop_size {
      let mut indices: Vec<usize> = (0..pop_size).filter(|&x| x != i).collect();

      // Shuffle and take 3
      let mut rng = rand::rng();
      for j in (1..indices.len()).rev() {
        let k = rng.random_range(0..=j);
        indices.swap(j, k);
      }
      let (a, b, c) = (indices[0], indices[1], indices[2]);

      // Create trial vector by applying DE mutation to polygons
      let base = &self.de_population[a].1;
      let diff1 = &self.de_population[b].1;
      let diff2 = &self.de_population[c].1;
      let target = &self.de_population[i].1;

      let trial = self.de_mutate_crossover(base, diff1, diff2, target);
      let rendered = render_image(&trial);
      let trial_score = sad_compare_raw(&self.target_pixels, &rendered.data);

      // Selection: keep better one
      if trial_score > self.de_population[i].0 {
        new_population.push((trial_score, trial));
      } else {
        new_population.push(self.de_population[i].clone());
      }
    }

    self.de_population = new_population;
    self
      .de_population
      .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
  }

  /// DE mutation and crossover: trial = base + F * (diff1 - diff2), then crossover with target
  fn de_mutate_crossover(
    &self,
    base: &Image,
    diff1: &Image,
    diff2: &Image,
    target: &Image,
  ) -> Image {
    let mut trial = base.clone();
    let mut rng = rand::rng();
    let f = self.config.mutation_factor;
    let cr = self.config.crossover_rate;

    // For each polygon, apply DE operations
    let min_len = trial
      .polygon
      .len()
      .min(diff1.polygon.len())
      .min(diff2.polygon.len())
      .min(target.polygon.len());

    for i in 0..min_len {
      if rng.random::<f32>() < cr {
        let poly = &mut trial.polygon[i];

        for c in 0..4 {
          let base_c = base.polygon[i].colour[c] as f32;
          let d1_c = diff1.polygon[i].colour[c] as f32;
          let d2_c = diff2.polygon[i].colour[c] as f32;
          let new_c = base_c + f * (d1_c - d2_c);
          poly.colour[c] = new_c.clamp(0.0, 255.0) as u8;
        }

        let min_points = poly
          .points
          .len()
          .min(diff1.polygon[i].points.len())
          .min(diff2.polygon[i].points.len());

        for p in 0..min_points {
          let base_x = base.polygon[i].points[p].0;
          let base_y = base.polygon[i].points[p].1;
          let d1_x = diff1.polygon[i].points[p].0;
          let d1_y = diff1.polygon[i].points[p].1;
          let d2_x = diff2.polygon[i].points[p].0;
          let d2_y = diff2.polygon[i].points[p].1;

          poly.points[p].0 = (base_x + f * (d1_x - d2_x)).clamp(0.0, self.width as f32 - 1.0);
          poly.points[p].1 = (base_y + f * (d1_y - d2_y)).clamp(0.0, self.height as f32 - 1.0);
        }
      } else {
        // Keep target's polygon
        trial.polygon[i] = target.polygon[i].clone();
      }
    }

    trial
  }

  fn get_best_buffer(&self) -> PixelBuffer {
    match self.config.algorithm_type {
      AlgorithmType::EvolutionStrategy => render_image(&self.es_parents[0].1),
      AlgorithmType::SimulatedAnnealing => render_image(self.sa_best.as_ref().unwrap()),
      AlgorithmType::DifferentialEvolution => render_image(&self.de_population[0].1),
    }
  }

  pub fn get_iteration(&self) -> usize {
    self.iteration
  }

  pub fn get_accuracy(&self) -> f32 {
    match self.config.algorithm_type {
      AlgorithmType::EvolutionStrategy => self.es_parents.first().map(|(s, _)| *s).unwrap_or(0.0),
      AlgorithmType::SimulatedAnnealing => self.sa_best_score,
      AlgorithmType::DifferentialEvolution => {
        self.de_population.first().map(|(s, _)| *s).unwrap_or(0.0)
      }
    }
  }

  pub fn is_finished(&self) -> bool {
    self.finished
  }

  pub fn get_dimensions(&self) -> (usize, usize) {
    (self.width, self.height)
  }
}
