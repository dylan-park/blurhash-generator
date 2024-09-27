use blurhash::encode;
use clap::Parser;
use image::GenericImageView;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input image file
    image: String,

    /// Number of components for X axis (1-9)
    #[arg(short = 'x', long, value_name = "NUM")]
    components_x: Option<u32>,

    /// Number of components for Y axis (1-9)
    #[arg(short = 'y', long, value_name = "NUM")]
    components_y: Option<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let components_x = cli.components_x.unwrap_or(4);
    let components_y = cli.components_y.unwrap_or(3);

    // Validate that if one component is specified, both must be
    if cli.components_x.is_some() != cli.components_y.is_some() {
        return Err("If specifying components, both -x and -y must be provided".into());
    }

    // Confirm values are within 1-9 range
    if !(1..=9).contains(&components_x) || !(1..=9).contains(&components_y) {
        return Err("The values of each component needs to be 1-9".into());
    }

    let img = image::open(Path::new(&cli.image))?;
    let (width, height) = img.dimensions();
    let pixels: Vec<u8> = img.to_rgba8().into_raw();

    let blurhash = encode(components_x, components_y, width, height, &pixels);
    println!("{}", blurhash.expect("Error during Blurhash encoding"));

    Ok(())
}
