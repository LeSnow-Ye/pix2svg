use anyhow::{Context, Result};
use clap::Parser;
use pix2svg::{convert_file_to_svg, save_svg_to_file, ConversionOptions};
use std::path::PathBuf;

/// Convert pixel art images to optimized SVG format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file path
    input: PathBuf,

    /// Output SVG file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pixel scale factor (1-1000)
    #[arg(short, long, default_value = "1", value_parser = clap::value_parser!(u32).range(1..=1000))]
    scale: u32,

    /// Minimum alpha threshold (0-255)
    #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8))]
    alpha_threshold: u8,

    /// Disable crisp edges rendering
    #[arg(long)]
    no_crisp_edges: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn process_image(args: &Args) -> Result<()> {
    if args.verbose {
        eprintln!("Loading image: {:?}", args.input);
    }

    // Validate input file exists
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", args.input);
    }

    if !args.input.is_file() {
        anyhow::bail!("Input path is not a file: {:?}", args.input);
    }

    // Configure conversion options
    let options = ConversionOptions {
        scale: args.scale,
        alpha_threshold: args.alpha_threshold,
        skip_transparent: true,
        crisp_edges: !args.no_crisp_edges,
    };

    if args.verbose {
        eprintln!("Conversion options:");
        eprintln!("  Scale: {}x", options.scale);
        eprintln!("  Alpha threshold: {}", options.alpha_threshold);
        eprintln!("  Crisp edges: {}", options.crisp_edges);
    }

    // Convert image to SVG
    let result = convert_file_to_svg(&args.input, options)
        .map_err(|e| anyhow::Error::msg(e.to_string()))
        .with_context(|| format!("Failed to convert image: {:?}", args.input))?;

    if args.verbose {
        eprintln!("Conversion results:");
        eprintln!(
            "  Image dimensions: {}x{}",
            result.image_dimensions.0, result.image_dimensions.1
        );
        eprintln!("  Generated rectangles: {}", result.rectangle_count);
        eprintln!("  SVG size: {} bytes", result.svg_size_bytes());
    }

    // Determine output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("svg");
        path
    });

    // Save SVG file
    save_svg_to_file(&result.svg_content, &output_path)
        .map_err(|e| anyhow::Error::msg(e.to_string()))
        .with_context(|| format!("Failed to save SVG file: {:?}", output_path))?;

    if args.verbose {
        eprintln!("SVG file saved: {:?}", output_path);
    } else {
        println!("Successfully converted to: {:?}", output_path);
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    process_image(&args).with_context(|| "Failed to process image")?;
    Ok(())
}
