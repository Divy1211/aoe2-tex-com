use image::{ImageBuffer, Rgba};
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyfunction]
#[pyo3(name = "render", signature = (
    main_layer, shadow_layer, shadow_layer_offset, player_color_mask, damage_mask = None, color = (255, 0, 0)
))]
pub fn render_frames(
    main_layer: &[u8],
    shadow_layer: &[u8],
    shadow_layer_offset: (i32, i32),
    player_color_mask: &[u8],
    damage_mask: Option<(&[u8], u32)>,
    color: (u8, u8, u8),
) -> PyResult<Vec<u8>> {
    let main_layer = image::load_from_memory(main_layer)
        .map_err(|e| PyValueError::new_err(format!("Failed to load shading image: {}", e)))?
        .to_rgba8();
    
    let shadow_layer = image::load_from_memory(shadow_layer)
        .map_err(|e| PyValueError::new_err(format!("Failed to load shadow image: {}", e)))?
        .to_luma8();
    
    let player_color_mask = image::load_from_memory(player_color_mask)
        .map_err(|e| PyValueError::new_err(format!("Failed to load mask image: {}", e)))?
        .to_luma8();

    let damage_mask = damage_mask.map(|(damage_mask, percent)| {
        image::load_from_memory(damage_mask)
            .map_err(|e| PyValueError::new_err(format!("Failed to load damage_mask image: {}", e)))
            .map(|img| (img.to_rgba8(), percent))
    }).transpose()?;
    
    let color_linear = [
        srgb_to_linear(color.0),
        srgb_to_linear(color.1),
        srgb_to_linear(color.2),
    ];

    let (main_width, main_height) = main_layer.dimensions();
    let (shadow_width, shadow_height) = shadow_layer.dimensions();
    
    let (width, height, main_off, shadow_off) = compute_offsets(
        (main_width, main_height), (shadow_width, shadow_height), shadow_layer_offset
    );
    
    let mut output = ImageBuffer::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let (mx, my) = offset((x, y), main_off);
            let (sx, sy) = offset((x, y), shadow_off);
            
            let (main_pixel, mask) = if mx < 0 || my < 0 || mx >= main_width as i32 || my >= main_height as i32 {
                (&Rgba([0, 0, 0, 0]), 0)
            } else {
                (main_layer.get_pixel(mx as u32, my as u32), player_color_mask.get_pixel(mx as u32, my as u32)[0])
            };
            let shadow = if sx < 0 || sy < 0 || sx >= shadow_width as i32 || sy >= shadow_height as i32 {
                0
            } else {
                shadow_layer.get_pixel(sx as u32, sy as u32)[0]  
            };
            
            let main_linear = [
                srgb_to_linear(main_pixel[0]),
                srgb_to_linear(main_pixel[1]),
                srgb_to_linear(main_pixel[2]),
            ];
            
            let luminance = (
                0.2126 * main_linear[0] +
                0.7152 * main_linear[1] +
                0.0722 * main_linear[2]
            ) * 5.0;
            
            let tinted = [
                luminance * color_linear[0],
                luminance * color_linear[1],
                luminance * color_linear[2],
            ];
            
            let mask = mask as f32 / 255.0;
            let blended = [
                mask * tinted[0] + (1.0 - mask) * main_linear[0],
                mask * tinted[1] + (1.0 - mask) * main_linear[1],
                mask * tinted[2] + (1.0 - mask) * main_linear[2],
            ];
            
            let final_rgb = [
                linear_to_srgb(blended[0]),
                linear_to_srgb(blended[1]),
                linear_to_srgb(blended[2]),
                if shadow == 0 { main_pixel[3] } else { shadow }
            ];
            
            output.put_pixel(x, y, Rgba([final_rgb[0], final_rgb[1], final_rgb[2], final_rgb[3]]));
        }
    }

    let mut png_bytes = Vec::new();
    output.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(png_bytes)
}

#[inline]
fn offset(
    (x, y): (u32, u32),
    (ox, oy): (i32, i32),
) -> (i32, i32) {
    (x as i32 - ox, y as i32 - oy)
}

#[inline]
fn compute_offsets(
    (mw, mh): (u32, u32),
    (sw, sh): (u32, u32),
    (ox, oy): (i32, i32),
) -> (u32, u32, (i32, i32), (i32, i32)) {
    let (sx1, sy1, sx2, sy2) = (ox, oy, ox + sw as i32, oy + sh as i32);
    let (x1, y1, x2, y2) = (0.min(sx1), 0.min(sy1), (mw as i32).max(sx2), (mh as i32).max(sy2));
    
    ((x2 - x1) as u32, (y2 - y1) as u32, (-x1, -y1), (sx1 - x1, sy1 - y1))
}

#[inline]
fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f32) -> u8 {
    let c = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}