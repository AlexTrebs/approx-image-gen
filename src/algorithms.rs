use image::RgbaImage;
use rand::distr::{Alphanumeric, SampleString};
use rand::{rng, Rng};

use crate::generations::generate_initial_image;
use crate::mutations::mutate_image;
use crate::renderer::render_image;
use crate::scoring::{score_images, CompareFn};
use crate::types::Image;

const MAX_ITERATIONS: usize = 100000;
const MUTATIONS_SCALING: usize = 10000;
const REQUIRED_ACCURACY: f32 = 0.95;
const CHILDREN_PER_PARENT: usize = 10;
const POPULATION_SIZE_INCREASE: usize = 1;
const POPULATION_SIZE_INCREASE_FREQUENCY: usize = 1000;
const MIN_MUTATIONS: usize = 2;
const KEEP_TOP: usize = 3;
const FILE_SAVE_FREQUENCY: usize = 1000;

pub fn strongest_mutates_alg(target: RgbaImage, compare_fn: CompareFn) -> Image {
  let (width, height) = target.dimensions();

  // Start with initial parents
  let mut parents: Vec<Image> = (0..KEEP_TOP)
    .map(|_| generate_initial_image(width as usize, height as usize))
    .collect();

  let mut best_score = 0.0;
  let mut iter_count: usize = 0;
  let mut no_improvement_count: usize = 0;

  while iter_count < MAX_ITERATIONS && best_score < REQUIRED_ACCURACY {
    // Generate children from all parents
    let mut children: Vec<Image> = Vec::new();

    for parent in parents.iter() {
      for _ in 0..CHILDREN_PER_PARENT {
        let mut child = parent.clone();
        let num_mutations = 1 + (iter_count / MUTATIONS_SCALING).min(MIN_MUTATIONS);

        for _ in 0..num_mutations {
          child = mutate_image(child);
        }

        children.push(child);
      }
    }

    children.extend(parents.clone());

    let mut scored = score_images(children, &target, compare_fn);
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let new_best_score = scored.first().map(|(s, _)| *s).unwrap_or(0.0);
    if new_best_score > best_score {
      best_score = new_best_score;
      no_improvement_count = 0;
    } else {
      no_improvement_count += 1;
    }

    // Take the two top performers as-is
    let top_two: Vec<Image> = scored
      .iter()
      .take(2)
      .map(|(_, img)| img.clone())
      .collect();

    // Take the worst performer and apply mutations
    let mut worst = scored.last().map(|(_, img)| img.clone()).unwrap();
    for _ in 0..5 {
      worst = mutate_image(worst);
    }

    parents = vec![top_two[0].clone(), top_two[1].clone(), worst];

    if iter_count % POPULATION_SIZE_INCREASE_FREQUENCY == 0 {
      // Increase population size to keep muliple of n best scoring parents
      parents.extend(
        scored
          .into_iter()
          .take(POPULATION_SIZE_INCREASE)
          .map(|(_, img)| img),
      )
    };

    if no_improvement_count > 500 {
      for parent in parents.iter_mut() {
        for _ in 0..5 {
          *parent = mutate_image(parent.clone());
        }
      }
      no_improvement_count = 0;
    }

    if iter_count % 500 == 0 {
      println!(
        "Iteration {}: accuracy = {:.4}% (stale: {})",
        iter_count,
        best_score * 100.0,
        no_improvement_count
      );
    }
    if iter_count % FILE_SAVE_FREQUENCY == 0 {
      let rendered = render_image(&parents[0]);
      let s: String = rng()
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
      rendered
        .save("./resources/output/".to_string() + &s + ".png")
        .unwrap();
    }

    iter_count += 1;
  }

  println!(
    "Finished after {} iterations with accuracy {:.4}%",
    iter_count,
    best_score * 100.0
  );

  parents.into_iter().next().unwrap()
}
