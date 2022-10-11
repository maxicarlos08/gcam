use crate::error::AppResult;
use epaint::ColorImage;
use image::{codecs::jpeg::JpegDecoder, DynamicImage};

/// Decodes a JPG image to a ColorImage
pub fn decode_image(image: &[u8]) -> AppResult<ColorImage> {
  let jpg_decoder = JpegDecoder::new(image)?;
  let image_data = DynamicImage::from_decoder(jpg_decoder)?.to_rgba8();

  Ok(ColorImage::from_rgba_unmultiplied(
    [image_data.width() as usize, image_data.height() as usize],
    image_data.as_flat_samples().as_slice(),
  ))
}
