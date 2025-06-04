# pix2svg

A fast and efficient command-line tool to convert pixel art images to optimized SVG format.

## Features

- **Optimized Output**: Uses greedy rectangle merging algorithm to minimize SVG file size
- **Multiple Formats**: Supports PNG, JPEG, GIF, BMP, and other common image formats
- **Transparency Support**: Handles transparent pixels with configurable alpha threshold
- **Pixel Scaling**: Scale up pixel art with crisp edges
- **Fast Processing**: Multi-threaded processing for large images
- **Production Ready**: Comprehensive error handling and logging

## Installation

### From Source

```bash
git clone https://github.com/yourusername/pix2svg.git
cd pix2svg
cargo build --release
```

The binary will be available at `target/release/pix2svg`.

### Using Cargo

```bash
cargo install pix2svg
```

## Usage

### Basic Usage

```bash
# Convert image.png to image.svg
pix2svg image.png

# Specify output file
pix2svg image.png -o output.svg

# Scale pixels by 4x
pix2svg image.png -s 4

# Enable verbose output
pix2svg image.png -v
```

### Advanced Options

```bash
# Convert with custom alpha threshold
pix2svg image.png --alpha-threshold 128

# Scale up and specify output
pix2svg sprite.png -s 8 -o large-sprite.svg -v
```

### Command Line Options

- `input`: Input image file path
- `-o, --output`: Output SVG file path (default: input filename with .svg extension)
- `-s, --scale`: Pixel scale factor (1-1000, default: 1)
- `--alpha-threshold`: Minimum alpha threshold for non-transparent pixels (0-255, default: 1)
- `--skip-transparent`: Skip transparent pixels (default: true)
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Show help information

## Algorithm

The tool uses a greedy rectangle merging algorithm similar to the one used in Aseprite:

1. **Scan pixels**: Process image pixel by pixel from top-left to bottom-right
2. **Find rectangles**: For each unprocessed pixel, find the largest rectangle of the same color
3. **Optimize**: Greedily expand rectangles horizontally and vertically
4. **Output**: Generate SVG `<rect>` elements for each rectangle

This approach significantly reduces the number of SVG elements compared to naive pixel-by-pixel conversion.

## Performance

The tool is optimized for pixel art and small images, but can handle larger images efficiently:

- **Small images** (≤64x64): Near-instantaneous processing
- **Medium images** (≤512x512): Sub-second processing
- **Large images** (≤2048x2048): Few seconds processing

## Examples

### Input Image (8x8 pixel art)

```
██  ██
██████
██  ██
██  ██
```

### Output SVG (with scale=4)

```svg
<svg version="1.1" width="32" height="32" xmlns="http://www.w3.org/2000/svg" shape-rendering="crispEdges">
  <rect x="0" y="0" width="8" height="32" fill="#000000" />
  <rect x="20" y="0" width="8" height="32" fill="#000000" />
  <rect x="8" y="8" width="12" height="8" fill="#000000" />
</svg>
```

Instead of 64 individual pixels, this produces just 3 rectangles!

## Supported Formats

### Input Formats

- PNG (recommended for pixel art)
- JPEG/JPG
- GIF
- BMP
- TIFF
- WEBP
- And more via the `image` crate

### Output Format

- SVG 1.1 with optimized rectangles

## Error Handling

The tool provides clear error messages for common issues:

- File not found
- Unsupported image format
- Permission errors
- Invalid parameters

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Algorithm inspired by Aseprite's SVG export functionality
- Built with Rust's excellent `image` and `clap` crates
