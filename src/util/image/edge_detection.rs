use image::{Rgba, RgbaImage};

#[rustfmt::skip]
const KERNEL_X: [f32; 9] = [
    -1.0, 0.0, 1.0, 
    -2.0, 0.0, 2.0, 
    -1.0, 0.0, 1.0
];

#[rustfmt::skip]
const KERNEL_Y: [f32; 9] = [
    -1.0, -2.0, -1.0, 
    0.0, 0.0, 0.0, 
    1.0, 2.0, 1.0
];

pub fn edge_detection(image: &RgbaImage) -> RgbaImage {
    let height = image.height();
    let width = image.width();

    let grayscale = image::imageops::grayscale(image);

    let mut edge_image = RgbaImage::new(width, height);
    let mut magnitude_buffer: Vec<f32> = vec![0.0; (width * height) as usize];
    let mut max_magnitude: f32 = 0.0;

    for y in 0..height {
        for x in 0..width {
            let mut gx: f32 = 0.0;
            let mut gy: f32 = 0.0;

            for ky in 0..3 {
                for kx in 0..3 {
                    let px = x as i32 + kx - 1;
                    let py = y as i32 + ky - 1;

                    let clamped_px = px.max(0).min(width as i32 - 1) as u32;
                    let clamped_py = py.max(0).min(height as i32 - 1) as u32;

                    let pixel_intensity = grayscale.get_pixel(clamped_px, clamped_py)[0] as f32;
                    let kernel_index = (ky * 3 + kx) as usize;

                    gx += pixel_intensity * KERNEL_X[kernel_index];
                    gy += pixel_intensity * KERNEL_Y[kernel_index];
                }
            }

            let magnitude = (gx.powi(2) + gy.powi(2)).sqrt();
            let buffer_index = (y * width + x) as usize;
            magnitude_buffer[buffer_index] = magnitude;

            if magnitude > max_magnitude {
                max_magnitude = magnitude;
            }
        }
    }

    if max_magnitude == 0.0 {
        max_magnitude = 1.0;
    }

    for y in 0..height {
        for x in 0..width {
            let buffer_index = (y * width + x) as usize;
            let normalized_magnitude =
                (magnitude_buffer[buffer_index] / max_magnitude * 255.0) as u8;

            edge_image.put_pixel(
                x,
                y,
                Rgba([
                    normalized_magnitude,
                    normalized_magnitude,
                    normalized_magnitude,
                    255,
                ]),
            );
        }
    }

    edge_image
}
