#[cfg(feature = "cli")]
mod algorithms;
#[cfg(feature = "cli")]
mod renderer;

mod generations;
mod mutations;
mod scoring;
mod types;

#[cfg(feature = "cli")]
use image::ImageReader;

#[cfg(feature = "cli")]
use algorithms::strongest_mutates_alg;
#[cfg(feature = "cli")]
use renderer::render_image;
#[cfg(feature = "cli")]
use scoring::sad_compare;

#[cfg(feature = "cli")]
fn main() {
    let img = ImageReader::open("./resources/rust.png")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();

    let result = strongest_mutates_alg(img, sad_compare);

    // Save the result
    let rendered = render_image(&result);
    rendered.save("./resources/output.png").unwrap();

    println!("Saved result to ./resources/output.png");
}

#[cfg(not(feature = "cli"))]
fn main() {
    // WASM builds don't use main
    println!("This binary requires the 'cli' feature. Use: cargo run --features cli");
}
