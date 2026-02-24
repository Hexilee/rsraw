use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use rsraw_sys as sys;

/// GPS information extracted from raw image EXIF data.
///
/// Latitude and longitude are stored as `[degrees, minutes, seconds]`.
/// Values are signed: negative latitude means South, negative longitude
/// means West, negative altitude means below sea level.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GpsInfo {
    /// Latitude as `[degrees, minutes, seconds]`. Negative for South.
    pub latitude: [f32; 3],
    /// Longitude as `[degrees, minutes, seconds]`. Negative for West.
    pub longitude: [f32; 3],
    /// GPS timestamp as `[hour, minute, second]`.
    pub gpstimestamp: [f32; 3],
    /// Altitude in meters. Negative for below sea level.
    pub altitude: f32,
}

/// Error returned when GPS data was not parsed by LibRaw.
#[derive(Debug, Clone, Copy)]
pub struct GpsParseError;

impl Display for GpsParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GPS data was not parsed by LibRaw")
    }
}

impl StdError for GpsParseError {}

impl GpsInfo {
    /// Try to convert from LibRaw GPS data.
    ///
    /// Returns `Err(GpsParseError)` if LibRaw did not parse any GPS data
    /// (i.e., `gpsparsed == 0`).
    ///
    /// On success, ref fields (latref, longref, altref) are folded into
    /// signed values: negative latitude means South, negative longitude
    /// means West, negative altitude means below sea level.
    pub fn try_from_raw(data: sys::libraw_gps_info_t) -> Result<Self, GpsParseError> {
        // gpsparsed is zero when LibRaw found no GPS EXIF data.
        // C `char` is signed on x86_64 but unsigned on aarch64/armv7,
        // so cast to u8 first for platform independence.
        if data.gpsparsed as u8 == 0 {
            return Err(GpsParseError);
        }

        let lat_sign = if data.latref as u8 == b'S' { -1.0 } else { 1.0 };
        let lon_sign = if data.longref as u8 == b'W' {
            -1.0
        } else {
            1.0
        };
        let alt_sign = if data.altref as u8 != 0 { -1.0 } else { 1.0 };

        Ok(Self {
            latitude: [
                data.latitude[0] * lat_sign,
                data.latitude[1] * lat_sign,
                data.latitude[2] * lat_sign,
            ],
            longitude: [
                data.longitude[0] * lon_sign,
                data.longitude[1] * lon_sign,
                data.longitude[2] * lon_sign,
            ],
            gpstimestamp: data.gpstimestamp,
            altitude: data.altitude * alt_sign,
        })
    }
}

impl From<sys::libraw_gps_info_t> for GpsInfo {
    fn from(data: sys::libraw_gps_info_t) -> Self {
        Self::try_from_raw(data).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a `libraw_gps_info_t` with the given values.
    fn make_raw_gps(
        lat: [f32; 3],
        lon: [f32; 3],
        alt: f32,
        latref: u8,
        longref: u8,
        altref: u8,
        gpsparsed: u8,
    ) -> sys::libraw_gps_info_t {
        sys::libraw_gps_info_t {
            latitude: lat,
            longitude: lon,
            altitude: alt,
            gpstimestamp: [12.0, 30.0, 45.0],
            latref: latref as _,
            longref: longref as _,
            altref: altref as _,
            gpsstatus: 0,
            gpsparsed: gpsparsed as _,
        }
    }

    #[test]
    fn try_from_raw_north_west() {
        let raw = make_raw_gps(
            [47.0, 36.0, 22.0],
            [122.0, 19.0, 55.0],
            56.0,
            b'N',
            b'W',
            0,
            1,
        );
        let gps = GpsInfo::try_from_raw(raw).unwrap();
        assert_eq!(gps.latitude, [47.0, 36.0, 22.0]);
        assert_eq!(gps.longitude, [-122.0, -19.0, -55.0]);
        assert_eq!(gps.altitude, 56.0);
        assert_eq!(gps.gpstimestamp, [12.0, 30.0, 45.0]);
    }

    #[test]
    fn try_from_raw_south_east() {
        let raw = make_raw_gps(
            [33.0, 51.0, 54.0],
            [151.0, 12.0, 36.0],
            10.0,
            b'S',
            b'E',
            0,
            1,
        );
        let gps = GpsInfo::try_from_raw(raw).unwrap();
        assert_eq!(gps.latitude, [-33.0, -51.0, -54.0]);
        assert_eq!(gps.longitude, [151.0, 12.0, 36.0]);
        assert_eq!(gps.altitude, 10.0);
    }

    #[test]
    fn try_from_raw_below_sea_level() {
        let raw = make_raw_gps(
            [31.0, 31.0, 0.0],
            [35.0, 28.0, 0.0],
            430.0,
            b'N',
            b'E',
            1, // below sea level
            1,
        );
        let gps = GpsInfo::try_from_raw(raw).unwrap();
        assert_eq!(gps.altitude, -430.0);
    }

    #[test]
    fn try_from_raw_not_parsed() {
        let raw = make_raw_gps(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            0,
            0,
            0,
            0, // gpsparsed = 0
        );
        assert!(GpsInfo::try_from_raw(raw).is_err());
    }

    #[test]
    fn from_not_parsed_returns_default() {
        let raw = make_raw_gps([0.0; 3], [0.0; 3], 0.0, 0, 0, 0, 0);
        let gps = GpsInfo::from(raw);
        assert_eq!(gps, GpsInfo::default());
    }

    #[test]
    fn from_parsed_applies_signs() {
        let raw = make_raw_gps(
            [47.0, 36.0, 22.0],
            [122.0, 19.0, 55.0],
            56.0,
            b'N',
            b'W',
            0,
            1,
        );
        let gps = GpsInfo::from(raw);
        // From delegates to try_from_raw, same result
        assert_eq!(gps.longitude, [-122.0, -19.0, -55.0]);
    }
}
