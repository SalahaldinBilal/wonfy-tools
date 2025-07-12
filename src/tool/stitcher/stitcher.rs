use std::{borrow::Cow, collections::VecDeque};

use image::{Pixel, RgbaImage, buffer::Pixels, imageops::rotate90};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::util::{
    image::{ImageCrop, edge_detection},
    iter::{IterWindows as _, padded_iter::PadExt as _},
};

use super::params::{CheckDirection, MatchMode, Order, OverlapScore, Position};
pub struct ImageStitcher {
    images: Vec<RgbaImage>,
    order: Order,
    direction: CheckDirection,
    window_size: usize,
    match_mode: MatchMode,
    crop: u32,
}

impl ImageStitcher {
    pub fn new(
        images: Vec<RgbaImage>,
        order: Order,
        direction: CheckDirection,
        window_size: usize,
        match_mode: MatchMode,
        crop: u32,
    ) -> Self {
        Self {
            images,
            order,
            direction,
            window_size,
            match_mode,
            crop,
        }
    }

    pub fn stitch(self) -> (RgbaImage, VecDeque<Position>) {
        let mut last_offset: Option<&Position> = None;
        let mut final_image: Option<RgbaImage> = None;
        let mut stitch_positions: VecDeque<Position> = VecDeque::new();

        match self.order {
            Order::Ordered => {
                for (image1, image2) in self.images.iter().tuple_windows::<(_, _)>() {
                    let image1 = final_image.as_ref().unwrap_or_else(|| image1);

                    let region = Self::find_stitch_region(
                        image1,
                        image2,
                        self.direction,
                        Order::Ordered,
                        self.window_size,
                        &self.match_mode,
                        self.crop,
                        last_offset,
                    );

                    let result = Self::stitch_images(
                        image1,
                        image2,
                        &region,
                        false,
                        self.crop,
                        self.direction,
                    );

                    Self::add_to_positions_ordered(
                        &mut stitch_positions,
                        region.position.clone(),
                        false,
                    );

                    last_offset = Some(stitch_positions.iter().next().unwrap());
                    final_image = Some(result);
                }
            }
            Order::Unordered => {
                let mut images: VecDeque<RgbaImage> = self.images.into();
                let mut stitch_positions_hash: VecDeque<(usize, Position)> = VecDeque::new();

                while images.len() > 1 {
                    let mut best_region = ([0, 1], OverlapScore::default());

                    for (index, image1) in images.iter().enumerate() {
                        for (index2, image2) in images.iter().enumerate().skip(index + 1) {
                            let region = Self::find_stitch_region(
                                image1,
                                image2,
                                self.direction,
                                Order::Unordered,
                                // &Default::default(),
                                self.window_size,
                                &self.match_mode,
                                self.crop,
                                None,
                            );

                            if region.score > best_region.1.score {
                                best_region = ([index, index2], region);
                            }
                        }
                    }

                    let (flipped, start, end) = match best_region.0[0] > best_region.0[1] {
                        true => (best_region.1.flipped, best_region.0[0], best_region.0[1]),
                        false => (!best_region.1.flipped, best_region.0[1], best_region.0[0]),
                    };

                    let image1 = images.remove(start).unwrap();
                    let image2 = images.remove(end).unwrap();
                    let stitched_image = Self::stitch_images(
                        &image1,
                        &image2,
                        &best_region.1,
                        flipped,
                        self.crop,
                        self.direction,
                    );

                    let new_image_id = stitched_image.as_ptr() as usize;
                    let image1_id = image1.as_ptr() as usize;
                    let image2_id = image2.as_ptr() as usize;

                    images.push_back(stitched_image);

                    Self::add_to_positions_unordered(
                        &mut stitch_positions_hash,
                        best_region.1.position.clone(),
                        flipped,
                        new_image_id,
                        (image1_id, image2_id),
                    );
                }

                final_image = Some(images.pop_back().unwrap());
                stitch_positions = stitch_positions_hash.into_iter().map(|(_, p)| p).collect();
            }
        }

        (final_image.expect("should be set by now"), stitch_positions)
    }

    fn add_to_positions_ordered(
        positions: &mut VecDeque<Position>,
        new_position: Position,
        flipped: bool,
    ) {
        if positions.len() == 0 {
            return positions.push_front(new_position);
        }

        match flipped {
            true => {
                for pos in positions.iter_mut() {
                    pos.x += new_position.x;
                    pos.y += new_position.y;
                }

                positions.push_back(new_position);
            }
            false => {
                positions.push_front(new_position);
            }
        }
    }

    fn add_to_positions_unordered(
        positions: &mut VecDeque<(usize, Position)>,
        new_position: Position,
        flipped: bool,
        result_ptr: usize,
        (id1, id2): (usize, usize),
    ) {
        let secondary_id = match flipped {
            true => id1,
            false => id2,
        };

        for (id, pos) in positions.iter_mut() {
            if *id != id1 && *id != id2 {
                continue;
            }

            if *id == secondary_id {
                *pos += &new_position;
            }

            *id = result_ptr;
        }

        positions.push_front((result_ptr, new_position));
    }

    fn stack_images_with_overlap(
        top_image: &RgbaImage,
        bottom_image: &RgbaImage,
        position: &Position,
        flipped: bool,
    ) -> RgbaImage {
        let (top_image, bottom_image) = match flipped {
            true => (bottom_image, top_image),
            false => (top_image, bottom_image),
        };

        let top_width = top_image.width();
        let top_height = top_image.height();
        let bottom_width = bottom_image.width();
        let bottom_height = bottom_image.height();

        let overlap_x_abs = position.x.unsigned_abs();
        let overlap_y_abs = position.y.unsigned_abs();

        assert!(overlap_y_abs < (bottom_height + top_height));
        assert!(overlap_x_abs < (bottom_width + top_width));

        let output_width = top_width + bottom_width;
        let output_height = top_height + bottom_height;

        let output_width = match position.x >= 0 {
            true => output_width - (top_width - overlap_x_abs),
            false => top_width + overlap_x_abs,
        };
        let output_height = match position.y >= 0 {
            true => output_height - (top_height - overlap_y_abs),
            false => top_height + overlap_y_abs,
        };

        let output_width = output_width.max(top_width);
        let output_height = output_height.max(top_height);

        let mut output_image = RgbaImage::new(output_width, output_height);

        let copy_first_x = if position.x >= 0 { 0 } else { overlap_x_abs };
        let copy_first_y = if position.y >= 0 { 0 } else { overlap_y_abs };

        for y in 0..top_height {
            for x in 0..top_width {
                let pixel = top_image.get_pixel(x, y);
                output_image.put_pixel(x + copy_first_x, y + copy_first_y, *pixel);
            }
        }

        let copy_second_x = if position.x >= 0 { overlap_x_abs } else { 0 };
        let copy_second_y = if position.y >= 0 { overlap_y_abs } else { 0 };

        for y in 0..bottom_height {
            for x in 0..bottom_width {
                let output_x = copy_second_x + x;
                let output_y = copy_second_y + y;

                if output_y < output_height && output_x < output_width {
                    let pixel = bottom_image.get_pixel(x, y);
                    output_image.put_pixel(output_x, output_y, *pixel);
                }
            }
        }

        output_image
    }

    #[inline(always)]
    fn pixel_as_value<P>(pixel: &P) -> u64
    where
        P: Pixel,
        P::Subpixel: Into<u64>,
    {
        let mut channels = pixel.channels();
        let mut num_of_channels = channels.len();

        if num_of_channels % 2 == 0 {
            channels = &channels[..num_of_channels - 1];
            num_of_channels -= 1;
        }

        channels
            .iter()
            .map(|p| <<P as Pixel>::Subpixel as Into<u64>>::into(*p))
            .sum::<u64>()
            / (num_of_channels as u64)
    }

    #[inline(always)]
    fn row_set_diff_score<'a, I1, I2, P>(
        first_set: I1,
        second_set: I2,
        padding: i32,
        window_size: u64,
    ) -> u64
    where
        P: Pixel + 'a,
        P::Subpixel: Into<u64>,
        I1: Iterator<Item = Pixels<'a, P>>,
        I2: Iterator<Item = Pixels<'a, P>>,
    {
        let mut sum = 0;

        for (row1, row2) in first_set.into_iter().zip(second_set) {
            sum += Self::row_diff_score(row1, row2, padding);
        }

        (u64::MAX - sum) / window_size
    }

    #[inline(always)]
    fn row_diff_score<P>(row1: Pixels<'_, P>, row2: Pixels<'_, P>, padding: i32) -> u64
    where
        P: Pixel,
        P::Subpixel: Into<u64>,
    {
        let mut score = 0;
        let first_iter = row1.map(Self::pixel_as_value);
        let second_iter = row2.map(Self::pixel_as_value);

        let first_iter = match padding {
            ..0 => first_iter.pad_start(0, (padding * -1) as usize),
            _ => first_iter.pad_end(0, padding as usize),
        };
        let second_iter = match padding {
            ..0 => second_iter.pad_end(0, (padding * -1) as usize),
            _ => second_iter.pad_start(0, padding as usize),
        };

        score += first_iter
            .zip(second_iter)
            .map(|(p1, p2)| p1.abs_diff(p2))
            .sum::<u64>();

        score
    }

    fn find_stitch_region(
        part1: &RgbaImage,
        part2: &RgbaImage,
        direction: CheckDirection,
        order: Order,
        // skip: &Position,
        window_size: usize,
        match_mode: &MatchMode,
        crop: u32,
        skip: Option<&Position>,
    ) -> OverlapScore {
        let (part1_check, part2_check) = match direction {
            CheckDirection::Vertical | CheckDirection::Sideways => (part1, part2),
            CheckDirection::Horizontal => (&rotate90(part1), &rotate90(part2)),
        };

        let (part1_check, part2_check) = match match_mode {
            MatchMode::Normal => (Cow::Borrowed(part1_check), Cow::Borrowed(part2_check)),
            MatchMode::Edges => (
                Cow::Owned(edge_detection(part1_check)),
                Cow::Owned(edge_detection(part2_check)),
            ),
        };

        let (skip_x, skip_y) = match skip {
            Some(skip) => match direction {
                CheckDirection::Horizontal => (i32::MIN, skip.x as usize),
                CheckDirection::Vertical | CheckDirection::Sideways => (skip.x, skip.y as usize),
            },
            None => (i32::MIN, 0),
        };

        let (horizontal_start, horizontal_move_end) = match direction {
            CheckDirection::Sideways => {
                let width = part2_check.width().max(part1_check.width());
                let start = (width - 1 - crop) as i32 * -1;

                println!("Skip X: {}", skip_x);
                println!("Skip Y: {}", skip_y);
                println!("horizontal_start: {}", start);

                (
                    if skip_x > start { skip_x } else { start },
                    (width - crop) as i32,
                )
            }
            _ => (0, 0),
        };

        let mut best_rows_to_merge = (vec![0 as isize; window_size], OverlapScore::default());

        // TODO:
        // 1. Figure out also make the vertical movement "start from negative"

        let second_start: Vec<_> = part2_check
            .rows()
            .skip(crop as usize)
            .windows(window_size)
            .next()
            .unwrap()
            .into();

        for horizontal_offset in horizontal_start..=horizontal_move_end {
            let min = part1_check
                .rows()
                .take((part1_check.height() - crop) as usize)
                .enumerate()
                .skip(skip_y)
                .map(|p| (p.0 as isize, p.1))
                .windows(window_size)
                .par_bridge()
                .map(|row| {
                    let (indices, row): (Vec<_>, Vec<_>) = row.into_iter().unzip();
                    let row = row.into_iter();
                    let row = match indices[0] < 0 {
                        true => itertools::Either::Left(row.rev()),
                        false => itertools::Either::Right(row),
                    };

                    let score = Self::row_set_diff_score(
                        row,
                        second_start.clone().into_iter(),
                        horizontal_offset,
                        window_size as u64,
                    );

                    let position = match direction {
                        CheckDirection::Vertical | CheckDirection::Sideways => Position {
                            y: indices[0] as i32,
                            x: horizontal_offset,
                        },
                        CheckDirection::Horizontal => Position {
                            x: indices[0] as i32,
                            y: 0,
                        },
                    };

                    (
                        indices,
                        OverlapScore {
                            score,
                            position,
                            flipped: false,
                        },
                    )
                })
                .max_by_key(|i| i.1.score)
                .expect("at least one row");

            if best_rows_to_merge.1.score < min.1.score {
                best_rows_to_merge = min;
            }
        }

        match order {
            Order::Ordered => best_rows_to_merge.1,
            Order::Unordered => {
                let mut overlap2 = Self::find_stitch_region(
                    part2,
                    part1,
                    direction,
                    Order::Ordered,
                    // skip,
                    window_size,
                    match_mode,
                    crop,
                    skip,
                );

                overlap2.flipped = true;

                if best_rows_to_merge.1.score > overlap2.score {
                    best_rows_to_merge.1
                } else {
                    overlap2
                }
            }
        }
    }

    pub fn stitch_images(
        part1: &RgbaImage,
        part2: &RgbaImage,
        position: &Position,
        flipped: bool,
        crop: u32,
        crop_direction: CheckDirection,
    ) -> RgbaImage {
        let crop = match crop_direction {
            CheckDirection::Vertical => ImageCrop {
                bottom: crop,
                ..Default::default()
            },
            CheckDirection::Horizontal => ImageCrop {
                right: crop,
                ..Default::default()
            },
            CheckDirection::Sideways => {
                let mut crop_obj = ImageCrop {
                    bottom: crop,
                    ..Default::default()
                };

                if position.x < 0 {
                    crop_obj.right = crop
                } else {
                    crop_obj.left = crop
                }

                crop_obj
            }
        };

        let part1 = crop.crop_image(part1);
        let part2 = crop.reverse().crop_image(part2);

        Self::stack_images_with_overlap(&part1, &part2, &position, flipped)
    }
}
