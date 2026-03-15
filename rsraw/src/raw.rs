use std::borrow::Cow;

use chrono::{DateTime, Local, TimeZone};
use rsraw_sys as sys;

use crate::{
    err::{Error, Result},
    processed::ProcessedImage,
    GpsInfo, LensInfo, ThumbnailImage, Thumbnails,
};

pub type BitDepth = u32;

pub const BIT_DEPTH_8: BitDepth = 8;
pub const BIT_DEPTH_16: BitDepth = 16;

pub struct RawImage {
    raw_data: *mut sys::libraw_data_t,
}

unsafe impl Sync for RawImage {}

unsafe impl Send for RawImage {}

impl RawImage {
    pub fn open(buf: &[u8]) -> Result<Self> {
        let raw_data = unsafe { sys::libraw_init(0) };
        Error::check(unsafe {
            sys::libraw_open_buffer(raw_data, buf.as_ptr() as *const _, buf.len())
        })?;
        Ok(Self { raw_data })
    }

    pub fn unpack(&mut self) -> Result<()> {
        unsafe {
            let raw_param = &mut (*self.raw_data).rawparams;
            raw_param.use_rawspeed = 1;
            raw_param.max_raw_memory_mb = 1024;
            Error::check(sys::libraw_unpack(self.raw_data))
        }
    }

    pub fn extract_thumbs(&mut self) -> Result<Vec<ThumbnailImage>> {
        let mut thumbs = Thumbnails::new();
        for i in 0..self.as_ref().thumbs_list.thumbcount {
            Error::check(unsafe { sys::libraw_unpack_thumb_ex(self.raw_data, i) })?;
            let thumb = &self.as_ref().thumbnail;
            let thumb_image = ThumbnailImage {
                format: thumb.tformat.into(),
                width: thumb.twidth as _,
                height: thumb.theight as _,
                colors: thumb.tcolors as _,
                data: unsafe {
                    std::slice::from_raw_parts(thumb.thumb as *const u8, thumb.tlength as _)
                        .to_vec()
                },
            };
            thumbs.append(thumb_image);
        }
        Ok(thumbs.into_inner())
    }

    pub fn width(&self) -> u32 {
        self.as_ref().sizes.width as _
    }

    pub fn height(&self) -> u32 {
        self.as_ref().sizes.height as _
    }

    pub fn pixels(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn colors(&self) -> i32 {
        self.as_ref().idata.colors
    }

    pub fn iso_speed(&self) -> u32 {
        self.as_ref().other.iso_speed as _
    }

    pub fn shutter(&self) -> f32 {
        self.as_ref().other.shutter as _
    }

    pub fn aperture(&self) -> f32 {
        self.as_ref().other.aperture as _
    }

    pub fn focal_len(&self) -> f32 {
        self.as_ref().other.focal_len as _
    }

    pub fn datetime(&self) -> Option<DateTime<Local>> {
        let ts = self.as_ref().other.timestamp;
        Local.timestamp_opt(ts, 0).single()
    }

    pub fn gps(&self) -> GpsInfo {
        self.as_ref().other.parsed_gps.into()
    }

    pub fn artist(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().other.artist[0] as *const _).to_string_lossy()
        }
    }

    pub fn desc(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().other.desc[0] as *const _).to_string_lossy()
        }
    }

    pub fn make(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().idata.make[0] as *const _).to_string_lossy()
        }
    }

    pub fn model(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().idata.model[0] as *const _).to_string_lossy()
        }
    }

    pub fn normalized_make(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().idata.normalized_make[0] as *const _)
                .to_string_lossy()
        }
    }

    pub fn normalized_model(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().idata.normalized_model[0] as *const _)
                .to_string_lossy()
        }
    }

    pub fn software(&self) -> Cow<'_, str> {
        unsafe {
            std::ffi::CStr::from_ptr(&self.as_ref().idata.software[0] as *const _).to_string_lossy()
        }
    }

    pub fn raw_count(&self) -> u32 {
        self.as_ref().idata.raw_count as _
    }

    pub fn dng_version(&self) -> u32 {
        self.as_ref().idata.dng_version as _
    }

    pub fn lens_info(&self) -> LensInfo {
        (&self.as_ref().lens).into()
    }

    pub fn full_info(&self) -> FullRawInfo {
        FullRawInfo {
            width: self.width(),
            height: self.height(),
            colors: self.colors(),
            iso_speed: self.iso_speed(),
            shutter: self.shutter(),
            aperture: self.aperture(),
            focal_len: self.focal_len(),
            datetime: self.datetime(),
            gps: self.gps(),
            artist: self.artist().to_string(),
            desc: self.desc().trim().into(),
            make: self.make().to_string(),
            model: self.model().to_string(),
            normalized_make: self.normalized_make().to_string(),
            normalized_model: self.normalized_model().to_string(),
            software: self.software().to_string(),
            raw_count: self.raw_count(),
            dng_version: self.dng_version(),
            lens_info: self.lens_info(),
        }
    }

    /// If possible, use the white balance from the camera.
    /// If not available, Auto-WB is used.
    pub fn set_use_camera_wb(&mut self, enable: bool) {
        unsafe {
            (*self.raw_data).params.use_camera_wb = if enable { 1 } else { 0 };
        }
    }

    /// On by default, call if you want to force use of DNG embedded matrix.
    pub fn set_use_camera_matrix(&mut self, enable: bool) {
        unsafe {
            (*self.raw_data).params.use_camera_matrix = if enable { 3 } else { 0 };
        }
    }

    pub fn process<const D: BitDepth>(&mut self) -> Result<ProcessedImage<D>> {
        debug_assert!(D == BIT_DEPTH_8 || D == BIT_DEPTH_16);
        unsafe { (*self.raw_data).params.output_bps = D as i32 };
        Error::check(unsafe { sys::libraw_dcraw_process(self.raw_data) })?;

        let mut result = 0i32;
        let processed = unsafe { sys::libraw_dcraw_make_mem_image(self.raw_data, &mut result) };
        Error::check(result)?;
        Ok(unsafe { ProcessedImage::from_raw(processed) })
    }
}

impl Drop for RawImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_close(self.raw_data) }
    }
}

impl AsRef<sys::libraw_data_t> for RawImage {
    fn as_ref(&self) -> &sys::libraw_data_t {
        unsafe { &*self.raw_data }
    }
}

impl AsMut<sys::libraw_data_t> for RawImage {
    fn as_mut(&mut self) -> &mut sys::libraw_data_t {
        unsafe { &mut *self.raw_data }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FullRawInfo {
    pub width: u32,
    pub height: u32,
    pub colors: i32,
    pub iso_speed: u32,
    pub shutter: f32,
    pub aperture: f32,
    pub focal_len: f32,
    pub datetime: Option<DateTime<Local>>,
    pub gps: GpsInfo,
    pub artist: String,
    pub desc: String,
    pub make: String,
    pub model: String,
    pub normalized_make: String,
    pub normalized_model: String,
    pub software: String,
    pub raw_count: u32,
    pub dng_version: u32,
    pub lens_info: LensInfo,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rsraw_sys::{
        LibRaw_camera_mounts_LIBRAW_MOUNT_Nikon_Z, LibRaw_camera_mounts_LIBRAW_MOUNT_Sony_E,
    };

    use super::*;
    use crate::{lens::FocusType, processed::ImageFormat, Mounts};

    fn get_test_assets_path() -> PathBuf {
        let root: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("must get manifest dir")
            .into();
        root.join("tests/assets")
    }

    #[test]
    fn test_raw_metadata() {
        let assets = get_test_assets_path();

        let test_cases = [
            (
                "test-z8.NEF",
                FullRawInfo {
                    width: 8280,
                    height: 5520,
                    colors: 3,
                    iso_speed: 250,
                    shutter: 1.0 / 100.0,
                    aperture: 3.5,
                    focal_len: 105.,
                    datetime: Local.with_ymd_and_hms(2024, 11, 4, 20, 11, 38).single(),
                    gps: Default::default(),
                    artist: "HEXILEE".into(),
                    desc: "".into(),
                    make: "Nikon".into(),
                    model: "Z 8".into(),
                    normalized_make: "Nikon".into(),
                    normalized_model: "Z 8".into(),
                    software: "Ver.02.00".into(),
                    raw_count: 1,
                    dng_version: 0,
                    lens_info: LensInfo {
                        min_focal: 105.0,
                        max_focal: 105.0,
                        max_aperture_at_min_focal: 2.8,
                        max_aperture_at_max_focal: 2.8,
                        lens_make: "NIKON".into(),
                        lens_name: "NIKKOR Z MC 105mm f/2.8 VR S".into(),
                        lens_serial: "20044280".into(),
                        internal_lens_serial: "".into(),
                        focal_length_in_35mm_format: 105,
                        mounts: Mounts::from(LibRaw_camera_mounts_LIBRAW_MOUNT_Nikon_Z).to_string(),
                        focus_type: FocusType::Prime,
                        feture_pre: "AF".into(),
                        feture_suf: "".into(),
                    },
                },
            ),
            (
                "test-a7rm4.ARW",
                FullRawInfo {
                    width: 9568,
                    height: 6376,
                    colors: 3,
                    iso_speed: 320,
                    shutter: 1.0 / 500.0,
                    aperture: 4.0,
                    focal_len: 40.,
                    datetime: Local.with_ymd_and_hms(2023, 11, 17, 13, 0, 13).single(),
                    gps: Default::default(),
                    artist: "hexilee".into(),
                    desc: "".into(),
                    make: "Sony".into(),
                    model: "ILCE-7RM4".into(),
                    normalized_make: "Sony".into(),
                    normalized_model: "ILCE-7RM4".into(),
                    software: "ILCE-7RM4 v1.20".into(),
                    raw_count: 1,
                    dng_version: 0,
                    lens_info: LensInfo {
                        min_focal: 40.0,
                        max_focal: 40.0,
                        max_aperture_at_min_focal: 1.4,
                        max_aperture_at_max_focal: 1.4,
                        lens_make: "".into(),
                        lens_name: "40mm F1.4 DG HSM | Art 018".into(),
                        lens_serial: "".into(),
                        internal_lens_serial: "".into(),
                        focal_length_in_35mm_format: 40,
                        mounts: Mounts::from(LibRaw_camera_mounts_LIBRAW_MOUNT_Sony_E).to_string(),
                        focus_type: FocusType::Prime,
                        feture_pre: "".into(),
                        feture_suf: "".into(),
                    },
                },
            ),
        ];
        for (file, expected) in test_cases {
            let path = assets.join(file);
            let data = std::fs::read(path).unwrap();
            let raw_image = RawImage::open(&data).expect("opened");
            assert_eq!(raw_image.raw_data.is_null(), false);
            let full_info = raw_image.full_info();
            assert_eq!(full_info, expected);
        }
    }

    #[test]
    fn test_thumbnails() {
        let assets = get_test_assets_path();
        let test_cases = [("test-z8.NEF",), ("test-a7rm4.ARW",)];
        for (file,) in test_cases {
            let path = assets.join(file);
            println!("{path:?}");
            let data = std::fs::read(path).unwrap();
            let mut raw_image = RawImage::open(&data).expect("opened");
            let thumbs = raw_image.extract_thumbs().expect("extracted");
            println!("{:?}", thumbs);
        }
    }

    #[test]
    fn test_processed() {
        let assets = get_test_assets_path();
        let test_cases = [
            (
                "test-z8.NEF",
                8280,
                5520,
                ImageFormat::Bitmap,
                3,
                16,
                274233600,
            ),
            (
                "test-a7rm4.ARW",
                9568,
                6376,
                ImageFormat::Bitmap,
                3,
                16,
                366033408,
            ),
        ];
        for (file, width, height, format, colors, bits, data_size) in test_cases {
            let path = assets.join(file);
            println!("{path:?}");
            let data = std::fs::read(path).unwrap();
            let mut raw_image = RawImage::open(&data).expect("opened");
            raw_image.unpack().expect("unpacked");
            let image = raw_image.process::<BIT_DEPTH_16>().expect("decoded");
            assert_eq!(image.width(), width);
            assert_eq!(image.height(), height);
            assert_eq!(image.image_format(), format);
            assert_eq!(image.colors(), colors);
            assert_eq!(image.bits(), bits);
            assert_eq!(image.data_size(), data_size);
        }
    }
}
