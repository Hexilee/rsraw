use rsraw_sys as sys;

use crate::Mounts;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LensInfo {
    pub min_focal: f32,
    pub max_focal: f32,
    pub max_aperture_at_min_focal: f32,
    pub max_aperture_at_max_focal: f32,
    pub lens_make: String,
    pub lens_name: String,
    pub lens_serial: String,
    pub internal_lens_serial: String,
    pub focal_length_in_35mm_format: u16,
    pub mounts: String,
    pub focus_type: FocusType,
    pub feture_pre: String,
    pub feture_suf: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusType {
    Unknown,
    Prime,
    Zoom,
    ZoomConstAp,
    ZoomVarAp,
}

impl From<sys::LibRaw_lens_focal_types> for FocusType {
    fn from(ft: sys::LibRaw_lens_focal_types) -> Self {
        match ft {
            sys::LibRaw_lens_focal_types_LIBRAW_FT_PRIME_LENS => Self::Prime,
            sys::LibRaw_lens_focal_types_LIBRAW_FT_ZOOM_LENS => Self::Zoom,
            sys::LibRaw_lens_focal_types_LIBRAW_FT_ZOOM_LENS_CONSTANT_APERTURE => Self::ZoomConstAp,
            sys::LibRaw_lens_focal_types_LIBRAW_FT_ZOOM_LENS_VARIABLE_APERTURE => Self::ZoomVarAp,
            _ => Self::Unknown,
        }
    }
}

impl<'a> From<&'a sys::libraw_lensinfo_t> for LensInfo {
    fn from(data: &'a sys::libraw_lensinfo_t) -> Self {
        let focus_type = match (data.makernotes.FocalType as sys::LibRaw_lens_focal_types).into() {
            FocusType::Unknown => {
                if data.MinFocal == data.MaxFocal {
                    FocusType::Prime
                } else if data.MaxAp4MinFocal == data.MaxAp4MaxFocal {
                    FocusType::ZoomConstAp
                } else {
                    FocusType::ZoomVarAp
                }
            }
            ft => ft,
        };

        Self {
            min_focal: data.MinFocal,
            max_focal: data.MaxFocal,
            max_aperture_at_min_focal: data.MaxAp4MinFocal,
            max_aperture_at_max_focal: data.MaxAp4MaxFocal,
            lens_make: unsafe {
                std::ffi::CStr::from_ptr(&data.LensMake[0] as *const _)
                    .to_string_lossy()
                    .to_string()
            },
            lens_name: unsafe {
                std::ffi::CStr::from_ptr(&data.Lens[0] as *const _)
                    .to_string_lossy()
                    .to_string()
            },
            lens_serial: unsafe {
                std::ffi::CStr::from_ptr(&data.LensSerial[0] as *const _)
                    .to_string_lossy()
                    .trim()
                    .to_owned()
            },
            internal_lens_serial: unsafe {
                std::ffi::CStr::from_ptr(&data.InternalLensSerial[0] as *const _)
                    .to_string_lossy()
                    .trim()
                    .to_owned()
            },
            focal_length_in_35mm_format: data.FocalLengthIn35mmFormat as _,
            mounts: Mounts::from(data.makernotes.LensMount as sys::LibRaw_camera_mounts)
                .to_string(),
            focus_type,
            feture_pre: unsafe {
                std::ffi::CStr::from_ptr(&data.makernotes.LensFeatures_pre as *const _)
                    .to_string_lossy()
                    .to_string()
            },
            feture_suf: unsafe {
                std::ffi::CStr::from_ptr(&data.makernotes.LensFeatures_suf as *const _)
                    .to_string_lossy()
                    .to_string()
            },
        }
    }
}
