use image::ImageFormat;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum EncodeFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
}

impl EncodeFormat {
    pub fn content_type(&self) -> &str {
        use EncodeFormat::*;

        match self {
            Png => "image/png",
            Jpeg => "image/jpeg",
            Gif => "image/gif",
            WebP => "image/webp",
        }
    }

    pub fn file_extension(&self) -> &str {
        use EncodeFormat::*;

        match self {
            Png => "png",
            Jpeg => "jpeg",
            Gif => "gif",
            WebP => "webp",
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn encode_format_content_type(format: EncodeFormat) -> String {
    format.content_type().into()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn encode_format_file_extension(format: EncodeFormat) -> String {
    format.file_extension().into()
}

impl Into<ImageFormat> for EncodeFormat {
    fn into(self) -> ImageFormat {
        use EncodeFormat::*;

        match self {
            Png => ImageFormat::Png,
            Jpeg => ImageFormat::Jpeg,
            Gif => ImageFormat::Gif,
            WebP => ImageFormat::WebP,
        }
    }
}
