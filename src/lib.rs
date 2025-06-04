//! # pix2svg
//!
//! A fast and efficient library to convert pixel art images to optimized SVG format.
//!
//! ## Features
//!
//! - **Optimized Output**: Uses greedy rectangle merging algorithm to minimize SVG file size
//! - **Multiple Formats**: Supports PNG, JPEG, GIF, BMP, and other common image formats via the `image` crate
//! - **Transparency Support**: Handles transparent pixels with configurable alpha threshold
//! - **Pixel Scaling**: Scale up pixel art with crisp edges
//! - **Fast Processing**: Optimized rectangle merging for efficient conversion
//!
//! ## Quick Start
//!
//! ```rust
//! use pix2svg::{convert_image_to_svg, ConversionOptions};
//! use image;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load an image
//! let img = image::open("pixel_art.png")?;
//!
//! // Configure conversion options
//! let options = ConversionOptions {
//!     scale: 4,
//!     alpha_threshold: 1,
//!     ..Default::default()
//! };
//!
//! // Convert to SVG
//! let svg_content = convert_image_to_svg(&img, options)?;
//!
//! // Save to file
//! std::fs::write("output.svg", svg_content)?;
//! # Ok(())
//! # }
//! ```

use image::{DynamicImage, Rgba, RgbaImage};

/// Configuration options for SVG conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Pixel scale factor (1-1000)
    pub scale: u32,
    /// Minimum alpha threshold for non-transparent pixels (0-255)
    pub alpha_threshold: u8,
    /// Skip transparent pixels (recommended for pixel art)
    pub skip_transparent: bool,
    /// Enable SVG shape-rendering="crispEdges" for pixel-perfect rendering
    pub crisp_edges: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            scale: 1,
            alpha_threshold: 1,
            skip_transparent: true,
            crisp_edges: true,
        }
    }
}

/// Represents a color with RGBA components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from an image RGBA pixel
    pub fn from_rgba(rgba: &Rgba<u8>) -> Self {
        Self::new(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    /// Check if the color is transparent based on threshold
    pub fn is_transparent(&self, threshold: u8) -> bool {
        self.a < threshold
    }

    /// Convert color to hex string (without alpha)
    pub fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Get opacity as a float (0.0 to 1.0)
    pub fn opacity(&self) -> f32 {
        self.a as f32 / 255.0
    }
}

/// Represents a rectangle with position, size, and color
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub color: Color,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: u32, y: u32, width: u32, height: u32, color: Color) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
        }
    }

    /// Convert rectangle to SVG rect element
    pub fn to_svg(&self, scale: u32) -> String {
        let mut svg = format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#{}" "##,
            self.x * scale,
            self.y * scale,
            self.width * scale,
            self.height * scale,
            self.color.to_hex()
        );

        if self.color.a != 255 {
            svg.push_str(&format!(r#"opacity="{:.3}" "#, self.color.opacity()));
        }

        svg.push_str("/>");
        svg
    }

    /// Get the area of the rectangle
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Internal image processor for rectangle extraction
struct ImageProcessor {
    image: RgbaImage,
    width: u32,
    height: u32,
    processed: Vec<Vec<bool>>,
    alpha_threshold: u8,
}

impl ImageProcessor {
    fn new(image: DynamicImage, alpha_threshold: u8) -> Self {
        let rgba_image = image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let processed = vec![vec![false; width as usize]; height as usize];

        Self {
            image: rgba_image,
            width,
            height,
            processed,
            alpha_threshold,
        }
    }

    fn get_pixel_color(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let pixel = self.image.get_pixel(x, y);
        let color = Color::from_rgba(pixel);

        if color.is_transparent(self.alpha_threshold) {
            None
        } else {
            Some(color)
        }
    }

    fn is_processed(&self, x: u32, y: u32) -> bool {
        self.processed[y as usize][x as usize]
    }

    fn mark_processed(&mut self, rect: &Rectangle) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.processed[y as usize][x as usize] = true;
            }
        }
    }

    fn find_max_rectangle(&self, start_x: u32, start_y: u32, color: Color) -> Rectangle {
        // Find maximum width for the current row
        let mut max_width = 0;
        for x in start_x..self.width {
            if self.is_processed(x, start_y) {
                break;
            }
            if let Some(pixel_color) = self.get_pixel_color(x, start_y) {
                if pixel_color != color {
                    break;
                }
                max_width = x - start_x + 1;
            } else {
                break;
            }
        }

        // Find maximum height with the current width
        let mut max_height = 0;
        for y in start_y..self.height {
            let mut can_extend = true;
            for x in start_x..start_x + max_width {
                if self.is_processed(x, y) {
                    can_extend = false;
                    break;
                }
                if let Some(pixel_color) = self.get_pixel_color(x, y) {
                    if pixel_color != color {
                        can_extend = false;
                        break;
                    }
                } else {
                    can_extend = false;
                    break;
                }
            }
            if !can_extend {
                break;
            }
            max_height = y - start_y + 1;
        }

        Rectangle::new(start_x, start_y, max_width, max_height, color)
    }

    fn extract_rectangles(&mut self) -> Vec<Rectangle> {
        let mut rectangles = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_processed(x, y) {
                    continue;
                }

                if let Some(color) = self.get_pixel_color(x, y) {
                    let rect = self.find_max_rectangle(x, y, color);
                    self.mark_processed(&rect);
                    rectangles.push(rect);
                } else {
                    // Mark transparent pixel as processed
                    self.processed[y as usize][x as usize] = true;
                }
            }
        }

        rectangles
    }
}

/// Conversion result containing SVG content and statistics
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// The generated SVG content
    pub svg_content: String,
    /// Number of rectangles generated
    pub rectangle_count: usize,
    /// Original image dimensions
    pub image_dimensions: (u32, u32),
}

impl ConversionResult {
    /// Get the length of the SVG content in bytes
    pub fn svg_size_bytes(&self) -> usize {
        self.svg_content.len()
    }
}

/// Convert a dynamic image to SVG format with given options
///
/// # Arguments
///
/// * `image` - The input image to convert
/// * `options` - Conversion options
///
/// # Returns
///
/// Returns a `ConversionResult` containing the SVG content and statistics
///
/// # Example
///
/// ```rust
/// use pix2svg::{convert_image_to_svg, ConversionOptions};
/// use image;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let img = image::open("test.png")?;
/// let options = ConversionOptions::default();
/// let result = convert_image_to_svg(&img, options)?;
/// println!("Generated {} rectangles", result.rectangle_count);
/// # Ok(())
/// # }
/// ```
pub fn convert_image_to_svg(
    image: &DynamicImage,
    options: ConversionOptions,
) -> Result<ConversionResult, Box<dyn std::error::Error>> {
    let width = image.width();
    let height = image.height();

    // Process image and extract rectangles
    let mut processor = ImageProcessor::new(image.clone(), options.alpha_threshold);
    let rectangles = processor.extract_rectangles();

    // Generate SVG content
    let svg_content = create_svg(&rectangles, width, height, &options);

    Ok(ConversionResult {
        svg_content,
        rectangle_count: rectangles.len(),
        image_dimensions: (width, height),
    })
}

/// Create SVG content from rectangles
fn create_svg(
    rectangles: &[Rectangle],
    width: u32,
    height: u32,
    options: &ConversionOptions,
) -> String {
    let mut svg = String::new();

    // XML declaration and SVG opening tag
    svg.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    svg.push('\n');

    let mut svg_tag = format!(
        r#"<svg version="1.1" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg""#,
        width * options.scale,
        height * options.scale
    );

    if options.crisp_edges {
        svg_tag.push_str(r#" shape-rendering="crispEdges""#);
    }

    svg_tag.push('>');
    svg.push_str(&svg_tag);
    svg.push('\n');

    // Add rectangles
    for rect in rectangles {
        svg.push_str(&rect.to_svg(options.scale));
        svg.push('\n');
    }

    // Closing tag
    svg.push_str("</svg>");
    svg.push('\n');

    svg
}

/// Convert image file path to SVG format
///
/// # Arguments
///
/// * `input_path` - Path to the input image file
/// * `options` - Conversion options
///
/// # Returns
///
/// Returns a `ConversionResult` containing the SVG content and statistics
pub fn convert_file_to_svg<P: AsRef<std::path::Path>>(
    input_path: P,
    options: ConversionOptions,
) -> Result<ConversionResult, Box<dyn std::error::Error>> {
    let image = image::open(input_path)?;
    convert_image_to_svg(&image, options)
}

/// Save SVG content to a file
///
/// # Arguments
///
/// * `svg_content` - The SVG content to save
/// * `output_path` - Path where to save the SVG file
pub fn save_svg_to_file<P: AsRef<std::path::Path>>(
    svg_content: &str,
    output_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(output_path, svg_content)?;
    Ok(())
}
