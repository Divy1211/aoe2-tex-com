use std::ptr;
use image::{ImageBuffer, Rgba};
use image_dds::{ImageFormat, Mipmaps, Quality, Surface};
use pyo3::{pyfunction, PyResult};
use pyo3::exceptions::PyValueError;
use crate::format::{BcFormat, BcQuality};
use crate::helper;

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (
    bytes, format = BcFormat::Bc1, quality = BcQuality::Slow, gen_commands = false, prev_info = None
))]
pub fn encode(
    bytes: &[u8],
    format: BcFormat,
    quality: BcQuality,
    gen_commands: bool,
    prev_info: Option<(Vec<u8>, u32, u32, i32, i32)>,
) -> PyResult<(Vec<u8>, Option<(Vec<(u8, u8)>, Vec<u8>)>)> {
    let image = image::load_from_memory(bytes).unwrap().to_rgba8();
    let surface = image_dds::SurfaceRgba8::from_image(&image).encode(
        match format {
            BcFormat::Bc1 => ImageFormat::BC1RgbaUnorm,
            BcFormat::Bc4 => ImageFormat::BC4RUnorm,
            BcFormat::Bc7 => ImageFormat::BC7RgbaUnorm,
        },
        match quality {
            BcQuality::Fast => Quality::Fast,
            BcQuality::Normal => Quality::Normal,
            BcQuality::Slow => Quality::Slow,
        },
        Mipmaps::Disabled,
    ).map_err(|e| PyValueError::new_err(e.to_string()))?;

    if !gen_commands {
        return Ok((surface.data, None));
    }
    
    let (width, height) = (surface.width, surface.height);

    let block_width = (width + 3) / 4;
    let block_height = (height + 3) / 4;

    let total_blocks = (block_width * block_height) as usize;

    let (_block_size, transparent_block) = match format {
        BcFormat::Bc1 => (8, vec![0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]),
        BcFormat::Bc4 => (8, vec![0_u8; 8]),
        BcFormat::Bc7 => {
            let mut block = vec![0_u8; 16];
            block[0] = 0x40;
            (16, block)
        },
    };

    let encoded_data = match format {
        BcFormat::Bc1 | BcFormat::Bc4 => {
            helper::encode_blocks::<u64>(&surface.data, block_width, total_blocks, &transparent_block, prev_info)
        }
        BcFormat::Bc7 => {
            helper::encode_blocks::<u128>(&surface.data, block_width, total_blocks, &transparent_block, prev_info)
        }
    };
    
    Ok((surface.data, Some(encoded_data)))
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (
    bytes, width, height, format, commands = None, prev_info = None
))]
pub fn decode(
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    format: BcFormat,
    commands: Option<Vec<(u8, u8)>>,
    prev_info: Option<(Vec<u8>, u32, u32, i32, i32)>,
) -> PyResult<(Vec<u8>, Vec<u8>)> {
    let data = if let Some(commands) = commands {
        let block_width = (width + 3) / 4;
        let block_height = (height + 3) / 4;

        let prev_info = prev_info.map(|(blocks, w, h, ox, oy)| {
            (blocks, (w + 3)/4, (h + 3)/4, ox/4, oy/4)
        });
        
        let total_blocks = (block_width * block_height) as usize;

        let (block_size, transparent_block) = match format {
            BcFormat::Bc1 => (8, vec![0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]),
            BcFormat::Bc4 => (8, vec![0_u8; 8]),
            BcFormat::Bc7 => {
                let mut block = vec![0_u8; 16];
                block[0] = 0x40;
                (16, block)
            },
        };

        let full_compressed_size = total_blocks * block_size;
        let mut full_compressed: Vec<u8> = Vec::with_capacity(full_compressed_size);

        let mut off = 0_usize;
        let mut block_idx = 0_usize;
        
        for (skip, draw) in commands {
            let start = full_compressed.len();
            let len = skip as usize * block_size;
            if start + len > full_compressed.capacity() {
                return Err(PyValueError::new_err(format!(
                    "Too many skip/draw calls ({} calls) for the specified dimensions ({width}x{height})",
                    (start + len)/8
                )))
            }

            if let Some((ref prev_blocks, prev_width, prev_height, off_x, off_y)) = prev_info {
                for i in 0..skip as usize {
                    let row = (block_idx + i) as i32 / block_width as i32 - off_y;
                    let col = (block_idx + i) as i32 % block_width as i32 - off_x;
                    
                    if row < 0 || col < 0 || row >= prev_height as i32 || col >= prev_width as i32 {
                        full_compressed.extend_from_slice(&transparent_block);
                    } else {
                        let pos = (row * prev_width as i32 + col) as usize * block_size;
                        if pos + block_size <= prev_blocks.len() {
                            full_compressed.extend_from_slice(&prev_blocks[pos..pos + block_size]);
                        } else {
                            return Err(PyValueError::new_err(format!(
                                "Previous block data ({} bytes) is too small for the specified dimensions ({prev_width}x{prev_height})",
                                prev_blocks.len()
                            )))
                        }
                    }
                }
            } else { unsafe {
                full_compressed.set_len(start + len);
                let mut dst = full_compressed.as_mut_ptr().add(start);

                for _ in 0..skip {
                    ptr::copy_nonoverlapping(
                        transparent_block.as_ptr(),
                        dst,
                        block_size,
                    );
                    dst = dst.add(block_size);
                }
            }}

            let len = draw as usize * block_size;
            if off+len > bytes.len() {
                return Err(PyValueError::new_err(format!(
                    "Block data ({} bytes) is too small for the specified dimensions ({width}x{height})",
                    bytes.len()
                )))
            }
            full_compressed.extend_from_slice(&bytes[off..off+len]);
            off += len;
            block_idx += skip as usize + draw as usize;
        }

        full_compressed
    } else {
        bytes
    };

    let surface = Surface {
        width,
        height,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: match format {
            BcFormat::Bc1 => ImageFormat::BC1RgbaUnorm,
            BcFormat::Bc4 => ImageFormat::BC4RUnorm,
            BcFormat::Bc7 => ImageFormat::BC7RgbaUnorm,
        },
        data: data.clone(),
    };

    let rgba_data = surface.decode_rgba8()
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .data;

    let image_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        width,
        height,
        rgba_data,
    ).ok_or_else(|| PyValueError::new_err("Failed to create image buffer"))?;

    let mut png_bytes = Vec::new();
    image_buffer.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok((png_bytes, data))
}