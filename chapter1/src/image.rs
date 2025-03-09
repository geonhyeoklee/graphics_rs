use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba, RgbaImage};

struct Image {
  width: u32,
  height: u32,
  channels: u8,
  pixels: Vec<[f32; 4]>,
}

const GAUSSIAN_WEIGHTS: [f32; 5] = [0.0545, 0.2442, 0.4026, 0.2442, 0.0545];

impl Image {
  fn read_from_file(filename: &str) -> Self {
    let img = ImageReader::open(filename)
      .expect("Failed to open file")
      .decode()
      .expect("Failed to decode image");

    let (width, height) = img.dimensions();

    let channels = match &img {
      DynamicImage::ImageLuma8(_) => 1,
      DynamicImage::ImageLumaA8(_) => 2,
      DynamicImage::ImageRgb8(_) => 3,
      DynamicImage::ImageRgba8(_) => 4,
      _ => panic!("Unsupported image format"),
    };

    let mut pixels = Vec::with_capacity((width * height) as usize);

    for (_, _, pixel) in img.pixels() {
      let rgba = match channels {
        3 => [
          pixel[0] as f32 / 255.0,
          pixel[1] as f32 / 255.0,
          pixel[2] as f32 / 255.0,
          1.0,
        ],
        4 => [
          pixel[0] as f32 / 255.0,
          pixel[1] as f32 / 255.0,
          pixel[2] as f32 / 255.0,
          pixel[3] as f32 / 255.0,
        ],
        _ => panic!("Unsupported channel count"),
      };
      pixels.push(rgba);
    }

    Self {
      width,
      height,
      channels,
      pixels,
    }
  }

  fn write_png(&self, filename: &str) {
    let mut img: RgbaImage = ImageBuffer::new(self.width, self.height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
      let index = (y * self.width + x) as usize;
      let rgba = self.pixels[index];

      *pixel = Rgba([
        (rgba[0] * 255.0) as u8,
        (rgba[1] * 255.0) as u8,
        (rgba[2] * 255.0) as u8,
        (rgba[3] * 255.0) as u8,
      ]);
    }

    img.save(filename).expect("Failed to save PNG file");
  }

  fn get_pixel(&self, i: i32, j: i32) -> &[f32; 4] {
    let i = i.clamp(0, self.width as i32 - 1);
    let j = j.clamp(0, self.height as i32 - 1);

    let index = (i + self.width as i32 * j) as usize;
    &self.pixels[index]
  }

  fn get_pixel_mut(&mut self, i: i32, j: i32) -> &mut [f32; 4] {
    let i = i.clamp(0, self.width as i32 - 1);
    let j = j.clamp(0, self.height as i32 - 1);

    let index = (i + self.width as i32 * j) as usize;
    &mut self.pixels[index]
  }
}
