use image::RgbaImage;

use crate::error::MissingFieldError;

use super::{CheckDirection, ImageStitcher, MatchMode, Order};

#[derive(Debug, Default)]
pub struct ImageStitcherBuilder {
    images: Option<Vec<RgbaImage>>,
    order: Option<Order>,
    direction: Option<CheckDirection>,
    window_size: Option<usize>,
    match_mode: Option<MatchMode>,
    crop: Option<u32>,
}

impl ImageStitcherBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    #[must_use]
    pub fn images<T: Into<Option<Vec<RgbaImage>>>>(self, images: T) -> Self {
        Self {
            images: images.into(),
            ..self
        }
    }

    #[must_use]
    pub fn order<T: Into<Option<Order>>>(self, order: T) -> Self {
        Self {
            order: order.into(),
            ..self
        }
    }

    #[must_use]
    pub fn direction<T: Into<Option<CheckDirection>>>(self, direction: T) -> Self {
        Self {
            direction: direction.into(),
            ..self
        }
    }

    #[must_use]
    pub fn window_size<T: Into<Option<usize>>>(self, window_size: T) -> Self {
        Self {
            window_size: window_size.into(),
            ..self
        }
    }

    #[must_use]
    pub fn match_mode<T: Into<Option<MatchMode>>>(self, match_mode: T) -> Self {
        Self {
            match_mode: match_mode.into(),
            ..self
        }
    }

    pub fn crop<T: Into<Option<u32>>>(self, crop: T) -> Self {
        Self {
            crop: crop.into(),
            ..self
        }
    }

    pub fn build(self) -> Result<ImageStitcher, MissingFieldError> {
        macro_rules! builder_field_unwrap {
            ($field: ident) => {
                self.$field
                    .ok_or_else(|| crate::error::MissingFieldError(stringify!($field).into()))?
            };
            ($field: ident, $default: literal) => {
                self.$field.unwrap_or_else(|| $default)
            };
        }

        Ok(ImageStitcher::new(
            builder_field_unwrap!(images),
            builder_field_unwrap!(order),
            builder_field_unwrap!(direction),
            builder_field_unwrap!(window_size),
            builder_field_unwrap!(match_mode),
            builder_field_unwrap!(crop, 0),
        ))
    }
}
