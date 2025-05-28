// Code effectively same as https://github.com/9elt/fast-dhash but without multithreading for wasm support

#[derive(Debug, Clone, Copy)]
pub struct DHash {
    pub hash: u64,
}

impl DHash {
    pub fn new(bytes: &[u8], width: u32, height: u32, channel_count: u8) -> Self {
        let width = width as usize;
        let height = height as usize;
        let channel_count = channel_count as usize;

        // NOTE: Very important, prevents possible segfault
        if width * height * channel_count != bytes.len() {
            panic!(
                "Invalid image dimensions, expected {} got {}",
                bytes.len(),
                width * height * channel_count
            );
        }

        let cell_width = width / 9;
        let cell_height = height / 8;

        let grid = if channel_count >= 3 {
            grid_from_rgb(bytes, width, cell_width, cell_height, channel_count)
        } else {
            grid_from_grayscale(bytes, width, cell_width, cell_height, channel_count)
        };

        let mut bits = [false; 64];

        for y in 0..8 {
            for x in 0..8 {
                bits[y * 8 + x] = grid[y][x] > grid[y][x + 1];
            }
        }

        let mut hash: u64 = 0;

        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                hash += 1 << i;
            }
        }

        Self { hash }
    }

    pub fn hamming_distance(&self, other: &Self) -> u32 {
        (self.hash ^ other.hash).count_ones()
    }
}

impl PartialEq for DHash {
    fn eq(&self, other: &Self) -> bool {
        self.hamming_distance(other) < 11
    }
}

fn grid_from_rgb(
    bytes: &[u8],
    width: usize,
    cell_width: usize,
    cell_height: usize,
    channel_count: usize,
) -> [[f64; 9]; 8] {
    let mut grid = [[0f64; 9]; 8];

    for y in 0..8 {
        let mut row = [0f64; 9];

        for (x, cell) in row.iter_mut().enumerate() {
            let from = x * cell_width;
            let to = from + cell_width;

            let mut rs = 0f64;
            let mut gs = 0f64;
            let mut bs = 0f64;

            for image_x in from..to {
                let from = y * cell_height;
                let to = from + cell_height;

                for image_y in from..to {
                    let i = (image_y * width + image_x) * channel_count;

                    unsafe {
                        rs += *bytes.get_unchecked(i) as f64;
                        gs += *bytes.get_unchecked(i + 1) as f64;
                        bs += *bytes.get_unchecked(i + 2) as f64;
                    }
                }
            }

            *cell += rs * 0.299 + gs * 0.587 + bs * 0.114;
        }

        grid[y] = row
    }

    grid
}

fn grid_from_grayscale(
    bytes: &[u8],
    width: usize,
    cell_width: usize,
    cell_height: usize,
    channel_count: usize,
) -> [[f64; 9]; 8] {
    let mut grid = [[0f64; 9]; 8];

    for y in 0..8 {
        let mut row = [0f64; 9];

        for (x, cell) in row.iter_mut().enumerate() {
            let from = x * cell_width;
            let to = from + cell_width;

            let mut luma = 0f64;

            for image_x in from..to {
                let from = y * cell_height;
                let to = from + cell_height;

                for image_y in from..to {
                    let i = (image_y * width + image_x) * channel_count;

                    unsafe {
                        luma += *bytes.get_unchecked(i) as f64;
                    }
                }
            }

            *cell += luma;
        }

        grid[y] = row;
    }

    grid
}
