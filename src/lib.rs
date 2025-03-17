use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::imageops::{resize, FilterType};
use image::{DynamicImage, ImageFormat};
use js_sys::{Reflect, Uint8Array};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

struct ImageConfig {
    format: String,                     // 输入图像格式
    crop: Option<CropConfig>,           // 裁剪参数
    size: Option<SizeConfig>,           // 缩放参数
    watermark: Option<WatermarkConfig>, // 水印参数
    output_format: Option<String>,      // 输出格式
    quality: Option<u8>,                // 输出质量（例如 JPEG）
}

struct CropConfig {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct SizeConfig {
    width: u32,
    height: u32,
}

struct WatermarkConfig {
    content: Vec<u8>,          // 水印图像数据
    position: [u32; 4],        // [x, y, width, height]
    opacity: f64,              // 水印透明度 (1 - 100)
    use_watermark_alpha: bool, // 是否使用水印自身的 Alpha 通道
}

impl ImageConfig {
    fn from_js_value(configs: &JsValue) -> Result<Self, JsError> {
        // 从 configs 中获取配置
        let format = Reflect::get(configs, &JsValue::from_str("format"))
            .map_err(|_| JsError::new("Failed to get 'format' from configs"))?
            .as_string()
            .ok_or(JsError::new("Input format must be a string"))?;

        // 验证 crop 是否是一个对象
        let crop = if let Some(crop_obj) = Reflect::get(configs, &JsValue::from_str("crop"))
            .map_err(|_| JsError::new("Failed to get 'crop' from configs"))?
            .dyn_ref::<js_sys::Object>()
        {
            let x = Reflect::get(crop_obj, &JsValue::from_str("x"))
                .map_err(|_| JsError::new("Failed to get 'x' from configs.crop"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            let y = Reflect::get(crop_obj, &JsValue::from_str("y"))
                .map_err(|_| JsError::new("Failed to get 'y' from configs.crop"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            let width = Reflect::get(crop_obj, &JsValue::from_str("width"))
                .map_err(|_| JsError::new("Failed to get 'width' from configs.crop"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            let height = Reflect::get(crop_obj, &JsValue::from_str("height"))
                .map_err(|_| JsError::new("Failed to get 'height' from configs.crop"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            Some(CropConfig {
                x,
                y,
                width,
                height,
            })
        } else {
            None
        };

        let size = if let Some(size_obj) = Reflect::get(configs, &JsValue::from_str("size"))
            .map_err(|_| JsError::new("Failed to get 'size' from configs"))?
            .dyn_ref::<js_sys::Object>()
        {
            let width = Reflect::get(size_obj, &JsValue::from_str("width"))
                .map_err(|_| JsError::new("Failed to get 'crop' from configs.size"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            let height = Reflect::get(size_obj, &JsValue::from_str("height"))
                .map_err(|_| JsError::new("Failed to get 'crop' from configs.height"))?
                .as_f64()
                .unwrap_or(0.0) as u32;
            Some(SizeConfig { width, height })
        } else {
            None
        };

        let watermark = if let Some(wm_obj) = Reflect::get(configs, &JsValue::from_str("watermark"))
            .map_err(|_| JsError::new("Failed to get 'watermark' from configs"))?
            .dyn_ref::<js_sys::Object>()
        {
            let content = Reflect::get(&wm_obj, &JsValue::from_str("content"))
                .map_err(|_| JsError::new("Failed to get 'content' from watermark"))?;
            let content_bytes = if let Some(content_array) = content.dyn_ref::<Uint8Array>() {
                content_array.to_vec()
            } else {
                return Err(JsError::new("'content' must be a Uint8Array"));
            };

            let position = Reflect::get(wm_obj, &JsValue::from_str("position"))
                .map_err(|_| JsError::new("Failed to get 'position' from configs.watermark"))?
                .dyn_into::<js_sys::Array>()
                .map(|arr| {
                    let x = arr.get(0).as_f64().unwrap_or(0.0) as u32;
                    let y = arr.get(1).as_f64().unwrap_or(0.0) as u32;
                    let width = arr.get(2).as_f64().unwrap_or(0.0) as u32;
                    let height = arr.get(3).as_f64().unwrap_or(0.0) as u32;
                    [x, y, width, height]
                })
                .unwrap_or([0, 0, 0, 0]);

            let opacity = Reflect::get(wm_obj, &JsValue::from_str("opacity"))
                .map_err(|_| JsError::new("Failed to get 'opacity' from configs.watermark"))?
                .as_f64()
                .unwrap_or(100.0);

            // 验证 opacity 是否在合理范围内
            if opacity < 0.0 || opacity > 100.0 {
                return Err(JsError::new("'opacity' must be between 0 and 100"));
            }

            // 归一化到 0.0-1.0
            let opacity = opacity / 100.0;

            let use_watermark_alpha =
                Reflect::get(wm_obj, &JsValue::from_str("use_watermark_alpha"))
                    .map_err(|_| {
                        JsError::new("Failed to get 'use_watermark_alpha' from configs.watermark")
                    })?
                    .as_bool()
                    .unwrap_or(false);

            Some(WatermarkConfig {
                content: content_bytes,
                position,
                opacity,
                use_watermark_alpha,
            })
        } else {
            None
        };

        let output_format = Reflect::get(configs, &JsValue::from_str("output_format"))
            .map_err(|_| JsError::new("Failed to get 'opacity' from configs.watermark"))?
            .as_string();

        let quality = Reflect::get(configs, &JsValue::from_str("quality"))
            .map_err(|_| JsError::new("Failed to get 'opacity' from configs.watermark"))?
            .as_f64()
            .map(|q| q as u8);

        Ok(ImageConfig {
            format,
            crop,
            size,
            watermark,
            output_format,
            quality,
        })
    }
}

fn apply_crop(mut img: DynamicImage, crop: &CropConfig) -> Result<DynamicImage, JsError> {
    if crop.x + crop.width > img.width() || crop.y + crop.height > img.height() {
        return Err(JsError::new("Crop dimensions exceed image bounds"));
    }
    Ok(img.crop(crop.x, crop.y, crop.width, crop.height))
}

fn apply_resize(img: &DynamicImage, size: &SizeConfig) -> DynamicImage {
    let resized = match img {
        DynamicImage::ImageRgba8(rgba) => {
            resize(rgba, size.width, size.height, FilterType::Lanczos3)
        }
        _ => resize(
            &img.to_rgba8(),
            size.width,
            size.height,
            FilterType::Lanczos3,
        ),
    };
    DynamicImage::ImageRgba8(resized)
}

fn apply_watermark(
    img: &DynamicImage,
    watermark: &WatermarkConfig,
) -> Result<DynamicImage, JsError> {
    let watermark_img = image::load_from_memory(&watermark.content)
        .map_err(|e| JsError::new(&format!("Failed to load watermark: {}", e)))?;

    if watermark.position.len() != 4 {
        return Err(JsError::new(
            "Watermark position must be an array of 4 numbers",
        ));
    }
    let [x, y, width, height] = watermark.position;
    if x + width > img.width() || y + height > img.height() {
        return Err(JsError::new("Watermark position exceeds image bounds"));
    }
    let resized_watermark = resize(&watermark_img, width, height, FilterType::Lanczos3);
    let watermark_rgba = DynamicImage::ImageRgba8(resized_watermark).to_rgba8();

    let mut img_rgba = match img {
        DynamicImage::ImageRgba8(rgba) => rgba.clone(),
        _ => img.to_rgba8(),
    };

    for (wx, wy, watermark_pixel) in watermark_rgba.enumerate_pixels() {
        let main_x = x + wx;
        let main_y = y + wy;
        if main_x < img_rgba.width() && main_y < img_rgba.height() {
            let main_pixel = img_rgba.get_pixel_mut(main_x, main_y);
            let alpha = if watermark.use_watermark_alpha {
                watermark_pixel[3]
            } else {
                (watermark_pixel[3] as f32 * watermark.opacity as f32) as u8
            };
            let alpha_f = alpha as f32 / 255.0;
            main_pixel[0] = (main_pixel[0] as f32 * (1.0 - alpha_f)
                + watermark_pixel[0] as f32 * alpha_f) as u8;
            main_pixel[1] = (main_pixel[1] as f32 * (1.0 - alpha_f)
                + watermark_pixel[1] as f32 * alpha_f) as u8;
            main_pixel[2] = (main_pixel[2] as f32 * (1.0 - alpha_f)
                + watermark_pixel[2] as f32 * alpha_f) as u8;
        }
    }
    Ok(DynamicImage::ImageRgba8(img_rgba))
}

fn encode_image(
    img: DynamicImage,
    format: ImageFormat,
    quality: Option<u8>,
) -> Result<Vec<u8>, JsError> {
    let mut buf = Cursor::new(Vec::new());
    match format {
        ImageFormat::Jpeg => {
            let quality = quality.unwrap_or(80);
            let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
            img.into_rgb8().write_with_encoder(encoder)?;
        }
        ImageFormat::Png => {
            let encoder = PngEncoder::new_with_quality(
                &mut buf,
                CompressionType::Best,
                image::codecs::png::FilterType::Paeth,
            );
            img.write_with_encoder(encoder)?;
        }
        ImageFormat::WebP => {
            let encoder = WebPEncoder::new_lossless(&mut buf);
            img.write_with_encoder(encoder)?;
        }
        _ => return Err(JsError::new("Unsupported output format")),
    }
    Ok(buf.into_inner())
}

#[wasm_bindgen]
pub fn image_cpr(input_data: &[u8], configs: &JsValue) -> Result<Vec<u8>, JsError> {
    // 解析配置
    let config = ImageConfig::from_js_value(configs)?;

    let format =
        ImageFormat::from_extension(&config.format).ok_or(JsError::new("Invalid input format"))?;

    // 加载图像
    let mut img = image::load_from_memory_with_format(input_data, format)?;

    // 应用裁剪
    if let Some(crop) = config.crop {
        img = apply_crop(img, &crop)?;
    }

    // 应用缩放
    if let Some(size) = config.size {
        img = apply_resize(&img, &size);
    }

    // 应用水印
    if let Some(watermark) = config.watermark {
        img = apply_watermark(&img, &watermark)?;
    }

    // 确定输出格式
    let output_format_str = config.output_format.unwrap_or(config.format);
    let output_format = ImageFormat::from_extension(&output_format_str)
        .ok_or(JsError::new("Invalid output format"))?;

    // 编码并返回
    encode_image(img, output_format, config.quality)
}
