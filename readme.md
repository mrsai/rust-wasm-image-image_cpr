# Image Processing with WebAssembly (WASM)

## Overview
This Rust module provides an image processing function that can be compiled to WebAssembly (WASM). It allows for cropping, resizing, watermarking, and encoding images in different formats with configurable quality settings.

## Dependencies
The module utilizes the following crates:
- `image`: For image decoding, processing, and encoding.
- `js_sys`: For interacting with JavaScript objects and `Uint8Array`.
- `wasm_bindgen`: For exposing Rust functions to JavaScript.

## Image Processing Configurations
The `ImageConfig` struct allows users to define the image processing parameters, including input format, cropping, resizing, watermarking, output format, and quality settings.

### ImageConfig Fields:
- `format` (String): The input image format.
- `crop` (Option<CropConfig>): Optional cropping parameters.
- `size` (Option<SizeConfig>): Optional resizing parameters.
- `watermark` (Option<WatermarkConfig>): Optional watermark parameters.
- `output_format` (Option<String>): Desired output format.
- `quality` (Option<u8>): Quality setting (e.g., for JPEG compression).

### CropConfig Fields:
- `x`, `y`: The top-left corner coordinates.
- `width`, `height`: The dimensions of the cropped area.

### SizeConfig Fields:
- `width`, `height`: The target dimensions for resizing.

### WatermarkConfig Fields:
- `content` (Vec<u8>): Watermark image data.
- `position` ([u32; 4]): The [x, y, width, height] of the watermark.
- `opacity` (f64): Transparency level (0-100 scaled to 0.0-1.0).
- `use_watermark_alpha` (bool): Whether to use the watermark's own alpha channel.

## Image Processing Functions

### `image_cpr(input_data: &[u8], configs: &JsValue) -> Result<Vec<u8>, JsError>`

Processes an input image with the given configurations and returns the processed image as a byte vector.

#### Steps:
1. Parses the configuration from a `JsValue`.
2. Loads the image from memory.
3. Applies cropping if specified.
4. Applies resizing if specified.
5. Applies watermarking if specified.
6. Encodes the image in the desired format and returns the processed image bytes.

## Utility Functions

### `apply_crop(img: DynamicImage, crop: &CropConfig) -> Result<DynamicImage, JsError>`
Crops the input image based on the provided dimensions.

### `apply_resize(img: &DynamicImage, size: &SizeConfig) -> DynamicImage`
Resizes the input image to the specified width and height using the Lanczos3 filter.

### `apply_watermark(img: &DynamicImage, watermark: &WatermarkConfig) -> Result<DynamicImage, JsError>`
Applies a watermark at the specified position with given opacity settings.

### `encode_image(img: DynamicImage, format: ImageFormat, quality: Option<u8>) -> Result<Vec<u8>, JsError>`
Encodes the processed image into the desired output format, supporting JPEG, PNG, and WebP.

## Usage Example (JavaScript)

```javascript
import init, { image_cpr } from '@/util/shared/pkg/rust_wasm_image_compress';

const imgData = await fetch('https://xxxx.jpg').then(res => res.arrayBuffer());
const wm = await fetch('https://xxxx.png').then(res => res.arrayBuffer());

const imageData = new Uint8Array(imgData); // Load image data as Uint8Array
const configs = {
  format: "png",
  crop: { x: 50, y: 50, width: 200, height: 200 },
  size: { width: 100, height: 100 },
  watermark: {
    content: new Uint8Array(wm), // Watermark image data
    position: [10, 10, 50, 50],
    opacity: 50,
    use_watermark_alpha: true
  },
  output_format: "jpeg",
  quality: 90
};

const processedImageData = image_cpr(imageData, configs);
```

## Error Handling
- Ensures input parameters are valid.
- Returns `JsError` with descriptive messages if operations fail.

## Conclusion
This module provides a robust image processing pipeline in Rust, making it accessible in JavaScript through WebAssembly, enabling efficient and high-performance image manipulation in web applications.
