use image::{
    DynamicImage, EncodableLayout, ImageBuffer, ImageFormat, ImageReader, ImageResult,
    PixelWithColorType, RgbaImage, imageops::FilterType,
};
use js_sys::{Array, Uint8Array};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::VecDeque, io::Cursor, ops::Deref, str::FromStr};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::tool::stitcher::{CheckDirection, ImageStitcherBuilder, MatchMode, Order, Position};

fn encode_image_as<P: image::Pixel, Container>(
    image: &ImageBuffer<P, Container>,
    format: ImageFormat,
) -> ImageResult<Vec<u8>>
where
    P: image::Pixel + PixelWithColorType,
    P: image::Pixel,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut bytes: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    image.write_to(&mut bytes, format)?;
    Ok(bytes.into_inner())
}

#[derive(Debug)]
pub enum PreviewData {
    Resize(f64),
    MaxWidth(u32),
    MaxHeight(u32),
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum PreviewType {
    Resize,
    MaxWidth,
    MaxHeight,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Preview {
    r#type: PreviewType,
    value: f64,
}

#[wasm_bindgen]
impl Preview {
    #[wasm_bindgen(constructor)]
    pub fn new(r#type: PreviewType, value: f64) -> Self {
        Self { r#type, value }
    }
}

impl Preview {
    pub fn get_value(self) -> PreviewData {
        use PreviewType::*;

        match self.r#type {
            Resize => PreviewData::Resize(self.value.min(1.0).max(0.0)),
            MaxWidth => PreviewData::MaxWidth(self.value as u32),
            MaxHeight => PreviewData::MaxHeight(self.value as u32),
        }
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct StitchedImage {
    image: Vec<u8>,
    stitch_positions: VecDeque<Position>,
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
impl StitchedImage {
    #[wasm_bindgen(
        js_name = "toJson",
        unchecked_return_type = "{ image: Uint8Array, stitchPositions: Array<{ x: number, y: number }>, width: number, height: number }"
    )]
    pub fn to_json(self) -> js_sys::Object {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("image"),
            &JsValue::from(self.image),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("stitchPositions"),
            &self
                .stitch_positions
                .into_iter()
                .map(|p| p.to_json())
                .collect::<Array>(),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("width"),
            &JsValue::from(self.width),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("height"),
            &JsValue::from(self.height),
        )
        .ok();

        obj
    }
}

impl StitchedImage {
    pub fn new(
        image: Vec<u8>,
        width: u32,
        height: u32,
        stitch_positions: VecDeque<Position>,
    ) -> Self {
        Self {
            width,
            height,
            image,
            stitch_positions,
        }
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct StitchReturn(StitchedImage, Option<StitchedImage>);

impl StitchReturn {
    pub fn new(image: StitchedImage, image_preview: Option<StitchedImage>) -> Self {
        Self(image, image_preview)
    }
}

#[wasm_bindgen]
impl StitchReturn {
    #[wasm_bindgen(unchecked_return_type = "[StitchedImage, StitchedImage?]")]
    pub fn images(self) -> js_sys::Array {
        let arr = js_sys::Array::new_with_length(2);
        arr.set(0, JsValue::from(self.0));
        arr.set(1, JsValue::from(self.1));
        arr
    }
}

#[wasm_bindgen]
pub fn stitch(
    images: Vec<Uint8Array>,
    direction: String,
    order: String,
    window_size: Option<usize>,
    match_mode: Option<String>,
    crop_padding: Option<u32>,
    preview: Option<Preview>,
) -> Result<StitchReturn, String> {
    let direction = CheckDirection::from_str(&direction).map_err(|e| format!("{:#?}", e))?;
    let order = Order::from_str(&order).map_err(|e| format!("{:#?}", e))?;

    let window_size = window_size.unwrap_or(6);
    let match_mode = match_mode
        .map(|s| MatchMode::from_str(&s).map_err(|e| format!("{:#?}", e)))
        .unwrap_or(Ok(MatchMode::Edges))?;

    let images: Vec<_> = images.into_iter().map(|u| u.to_vec()).collect();

    let images: Result<Vec<RgbaImage>, _> = images
        .par_iter()
        .map(|bytes| {
            Ok::<_, String>(
                ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()
                    .map_err(|e| format!("{:#?}", e))?
                    .decode()
                    .map_err(|e| format!("{:#?}", e))?
                    .to_rgba8(),
            )
        })
        .collect();

    let images = match images {
        Ok(images) => images,
        Err(err) => return Err(format!("Failed to load file: {:#?}", err)),
    };

    let stitcher = ImageStitcherBuilder::new()
        .images(images)
        .direction(direction)
        .order(order)
        .window_size(window_size)
        .match_mode(match_mode)
        .crop(crop_padding)
        .build()
        .map_err(|err| format!("{:#?}", err))?;

    let (final_image, stitch_positions) = stitcher.stitch();

    let stitched_image_data = encode_image_as(&final_image, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {:#?}", e))?;

    let stitched_image = StitchedImage::new(
        stitched_image_data,
        final_image.width(),
        final_image.height(),
        stitch_positions,
    );

    let preview_image = preview
        .map(|prev| {
            let prev = prev.get_value();

            let (width, height) = match prev {
                PreviewData::Resize(size) => (
                    (final_image.width() as f64 * size) as u32,
                    (final_image.height() as f64 * size) as u32,
                ),
                PreviewData::MaxHeight(height) => (final_image.width(), height),
                PreviewData::MaxWidth(width) => (width, final_image.height()),
            };

            let resized_image = encode_image_as(
                &DynamicImage::ImageRgba8(final_image)
                    .resize(width, height, FilterType::Lanczos3)
                    .to_rgba8(),
                ImageFormat::Png,
            )
            .ok();

            resized_image.map(|data| StitchedImage::new(data, width, height, Default::default()))
        })
        .flatten();

    Ok(StitchReturn::new(stitched_image, preview_image))
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    Ok(())
}
