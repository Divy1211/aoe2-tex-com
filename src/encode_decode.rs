use std::ptr;
use image::{ImageBuffer, Rgba};
use image_dds::{ImageFormat, Mipmaps, Quality, Surface};
use pyo3::{pyfunction, PyResult};
use pyo3::exceptions::PyValueError;
use crate::format::{BcFormat, BcQuality};

#[pyfunction]
pub fn encode(bytes: &[u8], format: BcFormat, quality: BcQuality) -> PyResult<Vec<u8>> {
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
    Ok(surface.data)
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (bytes, width, height, format, commands = None, prev_blocks = None, prev_width = None, prev_height = None, off_x = None, off_y = None))]
pub fn decode(
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    format: BcFormat,
    commands: Option<Vec<(u8, u8)>>,
    prev_blocks: Option<Vec<u8>>,
    prev_width: Option<u32>,
    prev_height: Option<u32>,
    off_x: Option<i32>,
    off_y: Option<i32>,
) -> PyResult<(Vec<u8>, Vec<u8>)> {
    let data = if let Some(commands) = commands {
        if prev_blocks.is_some() && (prev_width.is_none() || prev_height.is_none() || off_x.is_none() || off_y.is_none()) {
            return Err(PyValueError::new_err(
                "prev_blocks provided, but one of prev_width, prev_height, off_x, off_y was not provided"
            ));
        }
        
        let block_width = (width + 3) / 4;
        let block_height = (height + 3) / 4;

        let prev_width = prev_width.map(|w| (w + 3) / 4);
        let prev_height = prev_height.map(|h| (h + 3) / 4);
        let off_x = off_x.map(|x| x / 4);
        let off_y = off_y.map(|y| y / 4);
        
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

            if let Some(prev_blocks) = prev_blocks.as_ref() {
                let (prev_width, prev_height, off_x, off_y) = unsafe {
                    (
                        prev_width.unwrap_unchecked() as isize,
                        prev_height.unwrap_unchecked() as isize,
                        off_x.unwrap_unchecked() as isize,
                        off_y.unwrap_unchecked() as isize,
                    )
                };
                for i in 0..skip as usize {
                    let (row, col) = ((block_idx + i)/block_width as usize, (block_idx + i)%block_width as usize);
                    let (row, col) = (row as isize - off_y, col as isize - off_x);
                    if col >= prev_width || row >= prev_height || row < 0 || col < 0 {
                        full_compressed.extend_from_slice(&transparent_block);
                    } else {
                        let pos = (row * prev_width + col) as usize * block_size;
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