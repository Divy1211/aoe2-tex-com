use image::{ImageBuffer, Rgba};

pub fn fix_bc1_transparency(data: &mut [u8], image: &ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) {
    let block_width = (width + 3) / 4;
    let block_height = (height + 3) / 4;

    for block_y in 0..block_height {
        for block_x in 0..block_width {
            // Check which pixels in this 4x4 block are transparent
            let mut transparent_mask = 0u16; // 16 bits for 16 pixels
            let mut has_transparent = false;
            let mut has_opaque = false;

            for py in 0..4 {
                for px in 0..4 {
                    let x = block_x * 4 + px;
                    let y = block_y * 4 + py;
                    let pixel_idx = py * 4 + px;

                    if x < width && y < height {
                        let pixel = image.get_pixel(x, y);
                        if pixel[3] < 128 { // Treat alpha < 128 as transparent (1-bit alpha)
                            transparent_mask |= 1 << pixel_idx;
                            has_transparent = true;
                        } else {
                            has_opaque = true;
                        }
                    }
                }
            }

            // If block has any transparency, re-encode it properly
            if has_transparent {
                let block_index = (block_y * block_width + block_x) as usize;
                let byte_offset = block_index * 8;

                if !has_opaque {
                    // Fully transparent block
                    data[byte_offset..byte_offset + 8].copy_from_slice(&[0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
                } else {
                    // Partially transparent block - re-encode with alpha mode
                    encode_bc1_block_with_alpha(
                        &mut data[byte_offset..byte_offset + 8],
                        image,
                        block_x,
                        block_y,
                        width,
                        height,
                        transparent_mask,
                    );
                }
            }
        }
    }
}

fn encode_bc1_block_with_alpha(
    block: &mut [u8],
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    block_x: u32,
    block_y: u32,
    width: u32,
    height: u32,
    transparent_mask: u16,
) {
    // Collect opaque pixels to find best color palette
    let mut colors = Vec::new();

    for py in 0..4 {
        for px in 0..4 {
            let x = block_x * 4 + px;
            let y = block_y * 4 + py;
            let pixel_idx = py * 4 + px;

            if x < width && y < height {
                let pixel = image.get_pixel(x, y);
                // Only consider opaque pixels for color selection
                if (transparent_mask & (1 << pixel_idx)) == 0 {
                    colors.push([pixel[0], pixel[1], pixel[2]]);
                }
            }
        }
    }

    // Find two representative colors (simple min/max approach)
    let (color0_rgb, color1_rgb) = if colors.is_empty() {
        ([0u8, 0, 0], [0u8, 0, 0])
    } else if colors.len() == 1 {
        (colors[0], colors[0])
    } else {
        // Find min and max to get color range
        let mut min = colors[0];
        let mut max = colors[0];

        for color in &colors {
            for i in 0..3 {
                min[i] = min[i].min(color[i]);
                max[i] = max[i].max(color[i]);
            }
        }

        (min, max)
    };

    // Convert RGB888 to RGB565
    let color0_565 = rgb888_to_rgb565(color0_rgb);
    let color1_565 = rgb888_to_rgb565(color1_rgb);

    // Ensure color0 <= color1 for BC1 alpha mode (3-color + transparent)
    let (color0_565, color1_565) = if color0_565 > color1_565 {
        (color1_565, color0_565)
    } else {
        (color0_565, color1_565)
    };

    // Write color endpoints
    block[0] = (color0_565 & 0xFF) as u8;
    block[1] = ((color0_565 >> 8) & 0xFF) as u8;
    block[2] = (color1_565 & 0xFF) as u8;
    block[3] = ((color1_565 >> 8) & 0xFF) as u8;

    // Convert back to RGB888 for distance calculations
    let color0 = rgb565_to_rgb888(color0_565);
    let color1 = rgb565_to_rgb888(color1_565);

    // Calculate interpolated color (for 3-color mode: 1/2 blend)
    let color2 = [
        ((color0[0] as u16 + color1[0] as u16) / 2) as u8,
        ((color0[1] as u16 + color1[1] as u16) / 2) as u8,
        ((color0[2] as u16 + color1[2] as u16) / 2) as u8,
    ];

    // Encode indices
    let mut indices = 0u32;

    for py in 0..4 {
        for px in 0..4 {
            let x = block_x * 4 + px;
            let y = block_y * 4 + py;
            let pixel_idx = py * 4 + px;

            let index = if x >= width || y >= height {
                3 // Transparent for out of bounds
            } else if (transparent_mask & (1 << pixel_idx)) != 0 {
                3 // Transparent
            } else {
                // Find closest color among color0, color1, color2
                let pixel = image.get_pixel(x, y);
                let rgb = [pixel[0], pixel[1], pixel[2]];

                let dist0 = color_distance(rgb, color0);
                let dist1 = color_distance(rgb, color1);
                let dist2 = color_distance(rgb, color2);

                if dist0 <= dist1 && dist0 <= dist2 {
                    0
                } else if dist1 <= dist2 {
                    1
                } else {
                    2
                }
            };

            indices |= index << (pixel_idx * 2);
        }
    }

    // Write indices
    block[4] = (indices & 0xFF) as u8;
    block[5] = ((indices >> 8) & 0xFF) as u8;
    block[6] = ((indices >> 16) & 0xFF) as u8;
    block[7] = ((indices >> 24) & 0xFF) as u8;
}

fn rgb888_to_rgb565(rgb: [u8; 3]) -> u16 {
    let r = (rgb[0] as u16 >> 3) & 0x1F;
    let g = (rgb[1] as u16 >> 2) & 0x3F;
    let b = (rgb[2] as u16 >> 3) & 0x1F;
    (r << 11) | (g << 5) | b
}

fn rgb565_to_rgb888(rgb565: u16) -> [u8; 3] {
    let r = ((rgb565 >> 11) & 0x1F) as u8;
    let g = ((rgb565 >> 5) & 0x3F) as u8;
    let b = (rgb565 & 0x1F) as u8;

    // Expand to 8-bit by replicating high bits
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
    ]
}

fn color_distance(a: [u8; 3], b: [u8; 3]) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}