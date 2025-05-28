use std::{io::Cursor, ops::Deref};

use image::{EncodableLayout, ImageBuffer, ImageFormat, ImageResult, PixelWithColorType};

pub fn encode_image_as<P: image::Pixel, Container>(
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
