/// Produces the coordinates of top-left corners of tiles for the given image and tile dimensions
pub fn tiles(
    (image_width, image_height): (usize, usize),
    (tile_width, tile_height): (usize, usize),
) -> Vec<(usize, usize)> {
    let mut tiles = vec![];
    let mut y = 0;
    while y < image_height {
        let mut x = 0;
        while x < image_width {
            tiles.push((x, y));
            x += tile_width
        }
        y += tile_height;
    }
    tiles
}

/// Bytes per pixel; used in stride calculation for images
const BYTES_PER_PIXEL: usize = 3;

/// Blit from `src` to `(x, y)` in `dest` with the given dimensions of each. Assumes RGB at 8BPP
pub fn blit_rgb(
    src: &[u8],
    dest: &mut [u8],
    (x, y): (usize, usize),
    (dest_width, dest_height): (usize, usize),
    (src_width, src_height): (usize, usize),
) {
    debug_assert_eq!(src.len() % BYTES_PER_PIXEL, 0);

    debug_assert_eq!(src.len() % src_height, 0);
    debug_assert_eq!(dest.len() % dest_height, 0);

    debug_assert_eq!(src.len() % src_width, 0);
    debug_assert_eq!(dest.len() % dest_width, 0);

    debug_assert_eq!(src.len() / src_width, src_height * BYTES_PER_PIXEL);
    debug_assert_eq!(dest.len() / dest_width, dest_height * BYTES_PER_PIXEL);

    for (src_row, dest_row) in src
        .chunks_exact(BYTES_PER_PIXEL * src_width)
        .zip(dest.chunks_exact_mut(BYTES_PER_PIXEL * dest_width).skip(y))
    {
        let length_pixels = (x + src_width).min(dest_width) - x;

        dest_row[x * BYTES_PER_PIXEL..][..length_pixels * BYTES_PER_PIXEL]
            .copy_from_slice(&src_row[..length_pixels * BYTES_PER_PIXEL])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling() {
        let output_dims = (100, 200);
        let tile_dims = (33, 33);

        let mut output_data = vec![0; output_dims.0 * output_dims.1 * BYTES_PER_PIXEL];

        for pos in tiles(output_dims, tile_dims) {
            let (x, y) = dbg!(pos);
            let tile_data = (0..tile_dims.0 * tile_dims.1 * BYTES_PER_PIXEL)
                .map(|_| (x + y) as u8)
                .collect::<Vec<u8>>();

            blit_rgb(&tile_data, &mut output_data, pos, output_dims, tile_dims);
        }

        let mut expected_data = vec![0; output_dims.0 * output_dims.1 * BYTES_PER_PIXEL];

        for (y, row) in expected_data
            .chunks_exact_mut(output_dims.0 * BYTES_PER_PIXEL)
            .enumerate()
        {
            for (x, data) in row.iter_mut().enumerate() {
                let (tile_width, tile_height) = tile_dims;
                let x = ((x / BYTES_PER_PIXEL) / tile_width) * tile_width;
                let y = (y / tile_height) * tile_height;

                let val = (x + y) as u8;
                *data = val;
            }
        }

        assert_eq!(output_data, expected_data);
    }

    #[test]
    fn test_tiling_non_square() {
        let output_dims = (100, 350);
        let tile_dims = (33, 83);

        let mut output_data = vec![0; output_dims.0 * output_dims.1 * BYTES_PER_PIXEL];

        for pos in tiles(output_dims, tile_dims) {
            let (x, y) = dbg!(pos);
            let tile_data = (0..tile_dims.0 * tile_dims.1 * BYTES_PER_PIXEL)
                .map(|_| (x + y) as u8)
                .collect::<Vec<u8>>();

            blit_rgb(&tile_data, &mut output_data, pos, output_dims, tile_dims);
        }

        let mut expected_data = vec![0; output_dims.0 * output_dims.1 * BYTES_PER_PIXEL];

        for (y, row) in expected_data
            .chunks_exact_mut(output_dims.0 * BYTES_PER_PIXEL)
            .enumerate()
        {
            for (x, data) in row.iter_mut().enumerate() {
                let (tile_width, tile_height) = tile_dims;
                let x = ((x / BYTES_PER_PIXEL) / tile_width) * tile_width;
                let y = (y / tile_height) * tile_height;

                let val = (x + y) as u8;
                *data = val;
            }
        }

        assert_eq!(output_data, expected_data);
    }
}
