use crate::BcFormat;
use std::path::PathBuf;
use image::{ImageBuffer, Rgba};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct ProcessedFrame {
    #[pyo3(get)]
    image_bytes: Vec<u8>,
    #[pyo3(get)]
    offset_x1: u32,
    #[pyo3(get)]
    offset_y1: u32,
    #[pyo3(get)]
    offset_x2: u32,
    #[pyo3(get)]
    offset_y2: u32,
    #[pyo3(get)]
    anchor_x: i32,
    #[pyo3(get)]
    anchor_y: i32,
}

#[allow(clippy::type_complexity)]
#[pyfunction]
#[pyo3(name = "preprocess", signature = (frame_info, format = BcFormat::Bc1))]
pub fn preprocess_frames(
    frame_info: Vec<(PathBuf, (i32, i32))>,
    format: BcFormat,
) -> PyResult<(Vec<ProcessedFrame>, (u32, u32), (u32, u32))> {
    if frame_info.is_empty() {
        return Err(PyValueError::new_err("No frames provided"));
    }

    let mut processed_frames = Vec::with_capacity(frame_info.len());

    let mut max_dx1 = 0;
    let mut max_dy1 = 0;
    let mut max_dx2 = 0;
    let mut max_dy2 = 0;
    
    for (ref path, (anchor_x, anchor_y)) in frame_info.into_iter() {
        let img = image::open(path)
            .map_err(|e| PyValueError::new_err(format!(
                "Failed to load image '{}' due to '{}'", path.display(), e
            )))?
            .to_rgba8();

        let (width, height) = img.dimensions();
        let (x1, y1, x2, y2) = find_bounds(&img, width, height, &format);
        let (x1, y1, x2, y2) = pad_bounds(x1, y1, x2, y2, anchor_x, anchor_y);
        let (dx1, dy1, dx2, dy2) = get_detla(x1, y1, x2, y2, anchor_x, anchor_y);

        max_dx1 = max_dx1.max(dx1);
        max_dy1 = max_dy1.max(dy1);
        max_dx2 = max_dx2.max(dx2);
        max_dy2 = max_dy2.max(dy2);

        let anchor_x = anchor_x - x1;
        let anchor_y = anchor_y - y1;
        
        processed_frames.push((img, x1, y1, x2, y2, anchor_x, anchor_y));
    }

    let mut frame_infos = Vec::with_capacity(processed_frames.len());

    for (img, x1, y1, x2, y2, anchor_x, anchor_y) in processed_frames {
        let (width ,height) = (img.width() as i32, img.height() as i32);
        let (new_width, new_height) = (x2 - x1, y2 - y1);
        let mut processed_img = ImageBuffer::new(new_width as u32, new_height as u32);
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = x1 + x;
                let src_y = y1 + y;
                
                let pixel = if src_x < 0 || src_y < 0 || src_x >= width || src_y >= height {
                    &Rgba::from([0, 0, 0, 0])
                } else {
                    img.get_pixel(src_x as u32, src_y as u32)
                };
                
                processed_img.put_pixel(x as u32, y as u32, *pixel);
            }
        }
        
        let offset_x1 = max_dx1 - anchor_x;
        let offset_y1 = max_dy1 - anchor_y;
        let offset_x2 = offset_x1 + new_width;
        let offset_y2 = offset_y1 + new_height;
        
        let mut image_bytes = Vec::new();
        processed_img
            .write_to(&mut std::io::Cursor::new(&mut image_bytes), image::ImageFormat::Png)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        frame_infos.push(ProcessedFrame {
            image_bytes,
            offset_x1: offset_x1 as u32,
            offset_y1: offset_y1 as u32,
            offset_x2: offset_x2 as u32,
            offset_y2: offset_y2 as u32,
            anchor_x,
            anchor_y
        });
    }

    Ok((frame_infos, (max_dx1 as u32, max_dy1 as u32), ((max_dx1 + max_dx2) as u32, (max_dy1 + max_dy2) as u32)))
}

#[inline]
fn find_bounds(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32, format: &BcFormat
) -> (i32, i32, i32, i32) {
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    let mut has_content = false;

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let is_transparent = match format {
                BcFormat::Bc1 => pixel[3] < 128,
                BcFormat::Bc4 => pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0,
                BcFormat::Bc7 => pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 0,
            };
            if !is_transparent {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                has_content = true;
            }
        }
    }

    if !has_content {
        return (0, 0, 4, 4);
    }

    (min_x as i32, min_y as i32, (max_x + 1)  as i32, (max_y + 1) as i32)
}

#[inline]
fn get_detla(x1: i32, y1: i32, x2: i32, y2: i32, anchor_x: i32, anchor_y: i32) -> (i32, i32, i32, i32) {
    let dx1 = anchor_x - x1;
    let dy1 = anchor_y - y1;
    let dx2 = x2 - anchor_x;
    let dy2 = y2 - anchor_y;
    
    (dx1, dy1, dx2, dy2)
}

#[inline]
fn round_to_4(x: i32) -> i32 {
    ((x + 3) / 4) * 4
}

#[inline]
fn pad_bounds(x1: i32, y1: i32, x2: i32, y2: i32, anchor_x: i32, anchor_y: i32) -> (i32, i32, i32, i32) {
    let (dx1, dy1, dx2, dy2) = get_detla(x1, y1, x2, y2, anchor_x, anchor_y);

    let dx1 = round_to_4(dx1);
    let dy1 = round_to_4(dy1);
    let dx2 = round_to_4(dx2);
    let dy2 = round_to_4(dy2);
    
    (anchor_x - dx1, anchor_y - dy1, anchor_x + dx2, anchor_y + dy2)
}