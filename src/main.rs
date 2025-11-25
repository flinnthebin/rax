use anyhow::{Context, Result};
use image::{DynamicImage, ImageBuffer, Luma};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn main() {
  let mut buf = String::new();
  io::stdin().read_to_string(&mut buf).unwrap();
  let mut sc = Scanner::new(&buf);
  let n: String = sc.next();
  let path = Path::new(&n);
  let res = optimize_image_for_ocr(path);
  println!("{res:?}");
}

struct Scanner<'a> {
  it: std::str::SplitAsciiWhitespace<'a>,
}

impl<'a> Scanner<'a> {
  fn new(s: &'a str) -> Self {
    Self {
      it: s.split_ascii_whitespace(),
    }
  }
  fn next<T: std::str::FromStr>(&mut self) -> T {
    self.it.next().unwrap().parse().ok().unwrap()
  }
}

/// Takes a raw image path, enhances it for OCR, and returns the path to the temp file.
pub fn optimize_image_for_ocr(input_path: &Path) -> Result<PathBuf> {
  // 1. Open the image
  let img =
    image::open(input_path).with_context(|| format!("Failed to open image: {:?}", input_path))?;

  // 2. Convert to Grayscale (removes color noise from carbon paper)
  let gray_img = img.to_luma8();

  // 3. Apply Thresholding (Binarization)
  // This turns "dark grey text on light grey background" into "Black text on White background"
  // 128 is a standard middle point, but for carbon paper, we might tweak this later.
  let processed_img = apply_threshold(gray_img, 189);

  // 4. Save to a temporary file
  // Tesseract reads from disk, so we need to write this cleaned version down.
  let temp_dir = tempfile::tempdir()?;
  let temp_path = temp_dir.path().join("processed_receipt.png");

  processed_img
    .save(&temp_path)
    .context("Failed to save processed image")?;

  // We return the path. Note: The temp_dir will be deleted when the variable goes out of scope
  // in the caller, but for this MVP we persist the file or let the OS clean up /tmp later.
  // To keep it simple for now, we return a path that persists for the run duration.
  // In a real app, we'd manage the TempDir lifetime better.
  let persistent_path = std::env::temp_dir().join("receiptor_debug.png");
  processed_img.save(&persistent_path)?;

  Ok(persistent_path)
}

fn apply_threshold(
  buffer: ImageBuffer<Luma<u8>, Vec<u8>>,
  threshold: u8,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
  let mut out = ImageBuffer::new(buffer.width(), buffer.height());

  for (x, y, pixel) in buffer.enumerate_pixels() {
    let val = pixel[0];
    // If darker than threshold, make it BLACK (text). Else WHITE (paper).
    let new_val = if val < threshold { 0 } else { 255 };
    out.put_pixel(x, y, Luma([new_val]));
  }

  out
}
