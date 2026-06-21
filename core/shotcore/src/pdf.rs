//! Multi-page PDF export: embeds each capture as a JPEG image XObject, one image per page.
//! Pure-Rust PDF 1.4 writer (no external crate) with correct xref offsets and trailer.
use image::RgbaImage;

fn jpeg_bytes(img: &RgbaImage, quality: u8) -> Vec<u8> {
 let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
 let mut buf = Vec::new();
 let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality.clamp(1, 100));
 let _ = enc.encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8);
 buf
}

fn write_obj(buf: &mut Vec<u8>, offsets: &mut [usize], num: usize, body: &[u8]) {
 offsets[num] = buf.len();
 buf.extend_from_slice(format!("{} 0 obj\n", num).as_bytes());
 buf.extend_from_slice(body);
 buf.extend_from_slice(b"\nendobj\n");
}

/// Build a multi-page PDF (one image per page) as bytes. Each page size matches its image in
/// points (1 px = 1 pt). `quality` is the embedded-JPEG quality (1..=100).
pub fn images_to_pdf(images: &[RgbaImage], quality: u8) -> Vec<u8> {
 let n = images.len();
 let total = 2 + 3 * n;
 let mut offsets = vec![0usize; total + 1];
 let mut buf: Vec<u8> = Vec::new();
 buf.extend_from_slice(b"%PDF-1.4\n");
 write_obj(&mut buf, &mut offsets, 1, b"<< /Type /Catalog /Pages 2 0 R >>");
 let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + i * 3)).collect();
 let pages = format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), n);
 write_obj(&mut buf, &mut offsets, 2, pages.as_bytes());
 for (i, img) in images.iter().enumerate() {
 let (page, content, imgobj) = (3 + i * 3, 4 + i * 3, 5 + i * 3);
 let (w, h) = img.dimensions();
 let page_body = format!(
 "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /Im0 {} 0 R >> >> /Contents {} 0 R >>",
 w, h, imgobj, content
 );
 write_obj(&mut buf, &mut offsets, page, page_body.as_bytes());
 let stream = format!("q {} 0 0 {} 0 0 cm /Im0 Do Q", w, h);
 let content_body = format!("<< /Length {} >>\nstream\n{}\nendstream", stream.len(), stream);
 write_obj(&mut buf, &mut offsets, content, content_body.as_bytes());
 let jpg = jpeg_bytes(img, quality);
 offsets[imgobj] = buf.len();
 buf.extend_from_slice(format!("{} 0 obj\n", imgobj).as_bytes());
 buf.extend_from_slice(format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n", w, h, jpg.len()).as_bytes());
 buf.extend_from_slice(&jpg);
 buf.extend_from_slice(b"\nendstream\nendobj\n");
 }
 let xref_off = buf.len();
 buf.extend_from_slice(format!("xref\n0 {}\n", total + 1).as_bytes());
 buf.extend_from_slice(b"0000000000 65535 f \n");
 for num in 1..=total {
 buf.extend_from_slice(format!("{:010} 00000 n \n", offsets[num]).as_bytes());
 }
 buf.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF", total + 1, xref_off).as_bytes());
 buf
}

#[cfg(test)]
mod tests {
 use super::*;
 use image::Rgba;

 #[test]
 fn builds_valid_pdf() {
 let imgs = vec![
 RgbaImage::from_pixel(20, 20, Rgba([200, 50, 50, 255])),
 RgbaImage::from_pixel(30, 10, Rgba([50, 50, 200, 255])),
 ];
 let pdf = images_to_pdf(&imgs, 80);
 let s = String::from_utf8_lossy(&pdf);
 assert!(s.starts_with("%PDF-1."));
 assert!(s.contains("/Type /Catalog"));
 assert!(s.contains("/Type /Pages"));
 assert_eq!(s.matches("/Type /Page ").count(), 2);
 assert_eq!(s.matches("/Subtype /Image").count(), 2);
 assert!(s.trim_end().ends_with("%%EOF"));
 let pos = pdf.windows(9).rposition(|w| w == b"startxref").unwrap();
 let after = &pdf[pos + 10..];
 let num: String = after.iter().take_while(|&&b| b != 10u8).map(|&b| b as char).collect();
 let off: usize = num.trim().parse().unwrap();
 assert_eq!(&pdf[off..off + 4], b"xref");
 }

 #[test]
 fn empty_input_is_valid_pdf() {
 let pdf = images_to_pdf(&[], 80);
 let s = String::from_utf8_lossy(&pdf);
 assert!(s.starts_with("%PDF-1.") && s.trim_end().ends_with("%%EOF"));
 assert!(s.contains("/Count 0"));
 }
}
