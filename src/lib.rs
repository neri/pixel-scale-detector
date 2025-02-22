//! Pixel Scale Detector

#![cfg_attr(not(test), no_std)]

pub(crate) mod common_divisors;

// #[cfg(test)]
// mod tests;

extern crate alloc;

use common_divisors::find_common_divisors;
use core::num::NonZero;

pub fn get_pixel_scale(image: &ImageData, max_color_diff: Option<NonZero<u8>>) -> u32 {
    let mut possible_scales = find_common_divisors(image.width, image.height);
    if possible_scales.len() < 2 {
        return 0;
    }
    possible_scales.remove(0);

    for scale in possible_scales.iter().rev() {
        let Some(scale2) = NonZero::new(*scale as usize) else {
            unreachable!()
        };
        if image.is_matching_scale(scale2, scale2, max_color_diff) {
            return *scale;
        }
    }

    return 0;
}

#[unsafe(no_mangle)]
pub unsafe fn get_pixel_scale_ffi(
    data: *const u8,
    width: u32,
    height: u32,
    max_color_diff: u8,
) -> u32 {
    let Some(rgba_bytes) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
    else {
        return 0;
    };
    let rgba = unsafe { core::slice::from_raw_parts(data as *const RgbaUnion, rgba_bytes / 4) };
    let Some(width) = NonZero::new(width) else {
        return 0;
    };
    let Some(height) = NonZero::new(height) else {
        return 0;
    };
    let image = ImageData {
        width,
        height,
        rgba,
    };
    let max_color_diff = NonZero::new(max_color_diff);

    get_pixel_scale(&image, max_color_diff)
}

/// Detect the pixel scale of an image
///
/// # Image format
///
/// `[r:u8, g:u8, b:u8, a:u8, r, g, b, a, ...]`
pub fn get_pixel_scale_from_bytes(data: &[u8], width: u32, height: u32, max_color_diff: u8) -> u32 {
    let Some(rgba_bytes) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
    else {
        return 0;
    };
    if data.len() < rgba_bytes {
        return 0;
    }
    let rgba =
        unsafe { core::slice::from_raw_parts(data.as_ptr() as *const RgbaUnion, rgba_bytes / 4) };
    let Some(width) = NonZero::new(width) else {
        return 0;
    };
    let Some(height) = NonZero::new(height) else {
        return 0;
    };
    let image = ImageData {
        width,
        height,
        rgba,
    };
    let max_color_diff = NonZero::new(max_color_diff);

    get_pixel_scale(&image, max_color_diff)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RgbaUnion {
    rgba: RgbaComponents,
    u32: u32,
}

impl RgbaUnion {
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        unsafe { self.u32 }
    }

    #[inline]
    pub const fn as_components(&self) -> RgbaComponents {
        unsafe { self.rgba }
    }

    #[inline]
    pub fn is_matching_within_error(
        &self,
        other: Self,
        max_color_diff: Option<NonZero<u8>>,
    ) -> bool {
        self.as_components()
            .is_matching_within_error(other.as_components(), max_color_diff)
    }
}

impl PartialEq for RgbaUnion {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_u32() == other.as_u32()
    }
}

#[cfg(target_endian = "little")]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaComponents {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl RgbaComponents {
    #[inline]
    pub fn is_matching_within_error(
        &self,
        other: Self,
        max_color_diff: Option<NonZero<u8>>,
    ) -> bool {
        let r_diff = self.r.abs_diff(other.r);
        let g_diff = self.g.abs_diff(other.g);
        let b_diff = self.b.abs_diff(other.b);
        let a_diff = self.a.abs_diff(other.a);

        let max_diff = max_color_diff.map_or(0, |v| v.get());

        r_diff <= max_diff && g_diff <= max_diff && b_diff <= max_diff && a_diff <= max_diff
    }
}

impl From<RgbaComponents> for RgbaUnion {
    #[inline]
    fn from(components: RgbaComponents) -> Self {
        Self { rgba: components }
    }
}

pub struct ImageData<'a> {
    rgba: &'a [RgbaUnion],
    width: NonZero<u32>,
    height: NonZero<u32>,
}

impl ImageData<'_> {
    pub fn is_matching_scale(
        &self,
        scale_width: NonZero<usize>,
        scale_height: NonZero<usize>,
        max_color_diff: Option<NonZero<u8>>,
    ) -> bool {
        let rect_size = scale_width.checked_mul(scale_height).unwrap();
        let mut index_start = 0;
        let stride1 = self.width.get() as usize;
        let stride2 = stride1 * scale_height.get() as usize;
        if max_color_diff.is_none() {
            for _ in (0..self.height.get()).step_by(scale_height.get()) {
                let mut box_start = index_start;
                for _ in (0..self.width.get()).step_by(scale_width.get()) {
                    let mean = self.rgba[box_start];

                    let Some(row) = self.rgba.get(box_start..box_start + scale_width.get()) else {
                        return false;
                    };
                    if row.iter().skip(1).find(|v| **v != mean).is_some() {
                        return false;
                    }

                    let mut row_start = box_start;
                    for _ in 1..scale_height.get() {
                        let Some(row) = self.rgba.get(row_start..row_start + scale_width.get())
                        else {
                            return false;
                        };
                        if row.iter().find(|v| **v != mean).is_some() {
                            return false;
                        }
                        row_start += stride1;
                    }

                    box_start += scale_width.get();
                }
                index_start += stride2;
            }
        } else {
            for _ in (0..self.height.get()).step_by(scale_height.get()) {
                let mut box_start = index_start;
                for _ in (0..self.width.get()).step_by(scale_width.get()) {
                    let mut mean_r = 0;
                    let mut mean_g = 0;
                    let mut mean_b = 0;
                    let mut mean_a = 0;
                    let mut row_start = box_start;
                    for _ in 0..scale_height.get() {
                        let Some(row) = self.rgba.get(row_start..row_start + scale_width.get())
                        else {
                            return false;
                        };
                        for pixel in row.iter() {
                            let rgba = pixel.as_components();
                            mean_r += rgba.r as usize;
                            mean_g += rgba.g as usize;
                            mean_b += rgba.b as usize;
                            mean_a += rgba.a as usize;
                        }
                        row_start += stride1;
                    }
                    let mean = RgbaUnion::from(RgbaComponents {
                        r: (mean_r / rect_size.get()) as u8,
                        g: (mean_g / rect_size.get()) as u8,
                        b: (mean_b / rect_size.get()) as u8,
                        a: (mean_a / rect_size.get()) as u8,
                    });

                    let mut row_start = box_start;
                    for _ in 0..scale_height.get() {
                        let Some(row) = self.rgba.get(row_start..row_start + scale_width.get())
                        else {
                            return false;
                        };
                        for pixel in row.iter() {
                            if !mean.is_matching_within_error(*pixel, max_color_diff) {
                                return false;
                            }
                        }
                        row_start += stride1;
                    }

                    box_start += scale_width.get();
                }
                index_start += stride2;
            }
        }
        true
    }
}
