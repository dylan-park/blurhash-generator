use blurhash::encode;
use clap::Parser;
use image::GenericImageView;
use std::path::Path;
use reqwest::blocking::get;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input image file or URL
    image: String,

    /// Number of components for X axis (1-9)
    #[arg(short = 'x', long, value_name = "NUM")]
    components_x: Option<u32>,

    /// Number of components for Y axis (1-9)
    #[arg(short = 'y', long, value_name = "NUM")]
    components_y: Option<u32>,
}

fn looks_like_url(s: &str) -> bool {
    // First, check for common protocols
    if s.starts_with("http://") || s.starts_with("https://") {
        return true;
    }

    // Check for common URL patterns
    if s.starts_with("www.") {
        return true;
    }

    // Try to parse as URL with added https:// if needed
    let url_str = if !s.contains("://") {
        format!("https://{}", s)
    } else {
        s.to_string()
    };

    if let Ok(url) = Url::parse(&url_str) {
        // Check if it has a valid domain structure
        if url.has_host() && url.domain().is_some() {
            // Additional validation: should have at least one dot and valid TLD
            if let Some(domain) = url.domain() {
                return domain.contains('.') && 
                       !domain.ends_with('.') && 
                       !domain.contains('\\');  // Backslashes typically indicate local paths
            }
        }
    }
    false
}

fn looks_like_local_path(s: &str) -> bool {
    // Check for absolute paths
    if Path::new(s).is_absolute() {
        return true;
    }

    // Check for common path patterns
    if s.contains('\\') || s.contains('/') {
        // Check if it starts with drive letter (Windows)
        if s.len() >= 2 && s.chars().nth(1) == Some(':') {
            return true;
        }
        
        // Check for relative paths with directory separators
        if !s.contains("://") {
            return true;
        }
    }

    // Check for simple filenames with extensions
    if s.contains('.') && !s.contains("://") && !s.starts_with("www.") {
        let last_segment = Path::new(s).file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        // If it looks like a filename with extension
        if last_segment.contains('.') && !last_segment.starts_with('.') {
            return true;
        }
    }

    false
}

fn load_image(source: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    // If it looks like a URL and doesn't look like a local path
    if looks_like_url(source) && !looks_like_local_path(source) {
        // Handle cases where http(s):// is missing
        let url = if !source.contains("://") {
            format!("https://{}", source)
        } else {
            source.to_string()
        };

        let response = get(url)?;
        if !response.status().is_success() {
            return Err(format!("Failed to fetch image. Status: {}", response.status()).into());
        }
        let bytes = response.bytes()?;
        Ok(image::load_from_memory(&bytes)?)
    } else {
        // Treat as local path
        Ok(image::open(Path::new(source))?)
    }
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

    let img = load_image(&cli.image).map_err(|e| format!("Failed to load image: {}", e))?;
    let (width, height) = img.dimensions();
    let pixels: Vec<u8> = img.to_rgba8().into_raw();

    let blurhash = encode(components_x, components_y, width, height, &pixels);
    println!("{}", blurhash.expect("Error during Blurhash encoding"));

    Ok(())
}