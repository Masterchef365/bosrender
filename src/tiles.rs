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

/// Blit RGB tiles
pub fn blit_rgb(
    src: &[u8], 
    dest: &mut [u8], 
    (x, y): (usize, usize),
    (image_width, image_height): (usize, usize),
    (tile_width, tile_height): (usize, usize)
) {
    const BYTES_PER_PIXEL: usize = 3;
    for (src_row, dest_row) in src
        .chunks_exact(BYTES_PER_PIXEL * tile_width)
        .zip(
            dest
            .chunks_exact_mut(BYTES_PER_PIXEL * image_width)
            .skip(y)
        ) {
        let length_pixels = (x + tile_width).min(image_width) - x;

        dest_row[x * BYTES_PER_PIXEL..]
            [..length_pixels * BYTES_PER_PIXEL]
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

        let mut output_data = vec![0; output_dims.0 * output_dims.1 * 3];

        for pos in tiles(output_dims, tile_dims) {
            let (x, y) = dbg!(pos);
            let tile_data = (0..tile_dims.0 * tile_dims.1 * 3)
                .map(|_| (x + y) as u8)
                .collect::<Vec<u8>>();

            blit_rgb(
                &tile_data,
                &mut output_data,
                pos,
                output_dims,
                tile_dims,
            );
        }

        let mut expected_data = vec![0; output_dims.0 * output_dims.1 * 3];

        for (y, row) in expected_data.chunks_exact_mut(output_dims.0 * 3).enumerate() {
            for (x, data) in row.iter_mut().enumerate() {
                let (tile_width, tile_height) = tile_dims;
                let x = ((x / 3) / tile_width) * tile_width;
                let y = (y / tile_height) * tile_height;

                let val = (x + y) as u8;
                *data = val;
            }
        }

        assert_eq!(output_data, expected_data);
    }
}