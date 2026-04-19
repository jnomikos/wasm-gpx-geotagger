use std::io;

use bytes::Bytes;
use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use img_parts::webp::WebP;
use img_parts::ImageEXIF;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;

use crate::tagging::GpxPoint;

fn decimal_to_dms(decimal: f64) -> (u32, u32, f64) {
    let decimal = decimal.abs();
    let degrees = decimal.trunc() as u32;
    let minutes = (decimal.fract() * 60.0).trunc() as u32;
    let seconds = (decimal.fract() * 3600.0) % 60.0;
    (degrees, minutes, seconds)
}

pub fn get_file_extension(filename: &str) -> Option<FileExtension> {
    filename.rsplit('.').next().and_then(|ext| {
        match ext.to_lowercase().as_str() {
            "png" => Some(FileExtension::PNG { as_zTXt_chunk: false }),
            "jpg" | "jpeg" => Some(FileExtension::JPEG),
            "jxl" => Some(FileExtension::JXL),
            "tiff" | "tif" => Some(FileExtension::TIFF),
            "webp" => Some(FileExtension::WEBP),
            "heif" | "heic" => Some(FileExtension::HEIF),
            _ => None,
        }
    })
}

// Builds a Metadata object from raw TIFF bytes extracted by img-parts.
// Constructs a minimal synthetic JPEG (~few KB) so little_exif can parse
// the existing EXIF without seeing the full image buffer.
fn build_metadata_from_tiff(tiff: &Bytes) -> Result<Metadata, io::Error> {
    let seg_len = (2u16 + 6 + tiff.len() as u16).to_be_bytes();
    let mut buf = vec![0xFF, 0xD8, 0xFF, 0xE1]; // SOI + APP1 marker
    buf.extend_from_slice(&seg_len);
    buf.extend_from_slice(b"Exif\0\0");
    buf.extend_from_slice(tiff);
    buf.extend_from_slice(&[0xFF, 0xD9]); // EOI
    Metadata::new_from_vec(&buf, FileExtension::JPEG)
}

// Sets GPS tags on a Metadata object and returns the raw TIFF bytes.
// Uses JPEG encoding to extract raw TIFF — as_u8_vec(JPEG) produces
// [APP1 marker(2), len(2), "Exif\0\0"(6), tiff_data...], so skip 10 bytes.
// img-parts set_exif for all formats expects raw TIFF bytes.
fn build_gps_tiff(
    existing_exif: Option<Bytes>,
    lat_ref: &str,
    lon_ref: &str,
    lat_deg: u32, lat_min: u32, lat_sec: f64,
    lon_deg: u32, lon_min: u32, lon_sec: f64,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let mut metadata = match existing_exif {
        Some(tiff) => build_metadata_from_tiff(&tiff)?,
        None => Metadata::new(),
    };
    metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
    metadata.set_tag(ExifTag::GPSLatitude(vec![lat_deg.into(), lat_min.into(), lat_sec.into()]));
    metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));
    metadata.set_tag(ExifTag::GPSLongitude(vec![lon_deg.into(), lon_min.into(), lon_sec.into()]));
    let exif_app1 = metadata.as_u8_vec(FileExtension::JPEG)?;
    Ok(Bytes::copy_from_slice(&exif_app1[10..]))
}

pub fn tag_image(
    image_bytes: Vec<u8>,
    point: &GpxPoint,
    file_type: FileExtension,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let lat_ref = if point.lat >= 0.0 { "N" } else { "S" };
    let lon_ref = if point.lon >= 0.0 { "E" } else { "W" };
    let (lat_deg, lat_min, lat_sec) = decimal_to_dms(point.lat);
    let (lon_deg, lon_min, lon_sec) = decimal_to_dms(point.lon);

    match file_type {
        FileExtension::JPEG => {
            let mut jpeg = Jpeg::from_bytes(Bytes::from(image_bytes))?;
            let tiff = build_gps_tiff(jpeg.exif(), lat_ref, lon_ref, lat_deg, lat_min, lat_sec, lon_deg, lon_min, lon_sec)?;
            jpeg.set_exif(Some(tiff));
            Ok(jpeg.encoder().bytes())
        }
        FileExtension::PNG { .. } => {
            let mut png = Png::from_bytes(Bytes::from(image_bytes))?;
            let tiff = build_gps_tiff(png.exif(), lat_ref, lon_ref, lat_deg, lat_min, lat_sec, lon_deg, lon_min, lon_sec)?;
            png.set_exif(Some(tiff));
            Ok(png.encoder().bytes())
        }
        FileExtension::WEBP => {
            let mut webp = WebP::from_bytes(Bytes::from(image_bytes))?;
            let tiff = build_gps_tiff(webp.exif(), lat_ref, lon_ref, lat_deg, lat_min, lat_sec, lon_deg, lon_min, lon_sec)?;
            webp.set_exif(Some(tiff));
            Ok(webp.encoder().bytes())
        }
        _ => {
            let mut buf = image_bytes;
            let mut metadata = Metadata::new_from_vec(&buf, file_type)?;
            metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
            metadata.set_tag(ExifTag::GPSLatitude(vec![lat_deg.into(), lat_min.into(), lat_sec.into()]));
            metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));
            metadata.set_tag(ExifTag::GPSLongitude(vec![lon_deg.into(), lon_min.into(), lon_sec.into()]));
            metadata.write_to_vec(&mut buf, file_type)?;
            Ok(Bytes::from(buf))
        }
    }
}
