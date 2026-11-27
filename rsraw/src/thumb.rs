use std::fmt;

use rsraw_sys as sys;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThumbFormat {
    Unknown,
    Jpeg,
    Bitmap,
    Bitmap16,
    Layer,
    Rollei,
    H265,
    JpegXl,
}

#[derive(Debug, Default)]
pub struct Thumbnails {
    thumbs: Vec<ThumbnailImage>,
}

#[derive(Hash)]
pub struct ThumbnailImage {
    pub format: ThumbFormat,
    pub width: u32,
    pub height: u32,
    pub colors: u16,
    pub data: Vec<u8>,
}

impl Thumbnails {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append(&mut self, image: ThumbnailImage) {
        self.thumbs.push(image);
        self.thumbs.sort_by(|a, b| a.height.cmp(&b.height));
    }

    pub fn into_inner(self) -> Vec<ThumbnailImage> {
        self.thumbs
    }
}

impl From<sys::LibRaw_thumbnail_formats> for ThumbFormat {
    fn from(ft: sys::LibRaw_thumbnail_formats) -> Self {
        match ft {
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG => Self::Jpeg,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP => Self::Bitmap,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_BITMAP16 => Self::Bitmap16,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_LAYER => Self::Layer,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_ROLLEI => Self::Rollei,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_H265 => Self::H265,
            sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEGXL => Self::JpegXl,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Debug for ThumbnailImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThumbnailImage")
            .field("format", &self.format)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("colors", &self.colors)
            .field("data", &format_args!("{} bytes", self.data.len()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpegxl_thumbnail_format_is_mapped() {
        assert_eq!(
            ThumbFormat::from(sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEGXL),
            ThumbFormat::JpegXl
        );
    }
}
