use std::borrow::Cow;

use image::{ImageBuffer, Pixel, imageops::crop_imm};

#[derive(Debug, Clone, Default)]
pub struct ImageCrop {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl ImageCrop {
    pub fn reverse(self) -> Self {
        Self {
            top: self.bottom,
            bottom: self.top,
            left: self.right,
            right: self.top,
        }
    }

    pub fn crop_image<'a, P>(
        &self,
        image: &'a ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> Cow<'a, ImageBuffer<P, Vec<P::Subpixel>>>
    where
        P: Pixel + 'static,
    {
        if self.left | self.top | self.bottom | self.right == 0 {
            return Cow::Borrowed(image);
        }

        let (x, y, width, height) = (
            self.left,
            self.top,
            image.width() - self.right,
            image.height() - self.bottom,
        );

        Cow::Owned(crop_imm(image, x, y, width, height).to_image())
    }
}
