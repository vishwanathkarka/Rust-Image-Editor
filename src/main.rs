use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use imageproc::{
    geometric_transformations::{rotate_about_center, Interpolation},
    filter::{gaussian_blur_f32},
};
#[derive(Debug)]
pub enum ImageError {
    LoadError(String),
    OperationError(String),
}

pub struct ImageProcessor {
    image: DynamicImage,
}

impl ImageProcessor {
    /// Create a new ImageProcessor from a file path
    pub fn new(path: &str) -> Result<Self, ImageError> {
        image::open(path)
            .map(|img| ImageProcessor { image: img })
            .map_err(|e| ImageError::LoadError(e.to_string()))
    }

    /// Create from an existing DynamicImage
    pub fn from_dynamic_image(image: DynamicImage) -> Self {
        ImageProcessor { image }
    }

    /// Crop the image given coordinates
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<&mut Self, ImageError> {
        if x + width > self.image.width() || y + height > self.image.height() {
            return Err(ImageError::OperationError(
                "Crop dimensions exceed image bounds".to_string(),
            ));
        }

        self.image = self.image.crop(x, y, width, height);
        Ok(self)
    }

    /// Rotate the image by the specified angle in degrees
    pub fn rotate(&mut self, angle: f32) -> Result<&mut Self, ImageError> {
        // Convert angle to radians
        let radians = angle.to_radians();
        
        // Use rotate_about_center from imageproc
        self.image = DynamicImage::ImageRgba8(
            rotate_about_center(
                &self.image.to_rgba8(),
                radians,
                Interpolation::Bilinear,
                Rgba([0, 0, 0, 0])
            )
        );
        
        Ok(self)
    }


    pub fn adjust_brightness(&mut self, factor: f32) -> Result<&mut Self, ImageError> {
        let mut img = self.image.to_rgba8();
        for pixel in img.pixels_mut() {
            pixel[0] = (pixel[0] as f32 * factor).min(255.0) as u8;
            pixel[1] = (pixel[1] as f32 * factor).min(255.0) as u8;
            pixel[2] = (pixel[2] as f32 * factor).min(255.0) as u8;
        }
        self.image = DynamicImage::ImageRgba8(img);
        Ok(self)
    }

    pub fn blur(&mut self, sigma: f32) -> Result<&mut Self, ImageError> {
        let img = self.image.to_rgba8();
        let blurred = gaussian_blur_f32(&img, sigma);
        self.image = DynamicImage::ImageRgba8(blurred);
        Ok(self)
    }

    pub fn grayscale(&mut self) -> Result<&mut Self, ImageError> {
        self.image = DynamicImage::ImageRgba8(self.image.grayscale().to_rgba8());
        Ok(self)
    }

     /// Invert the colors of the image
     pub fn invert(&mut self) -> Result<&mut Self, ImageError> {
        let mut img = self.image.to_rgba8();
        for pixel in img.pixels_mut() {
            pixel[0] = 255 - pixel[0];
            pixel[1] = 255 - pixel[1];
            pixel[2] = 255 - pixel[2];
        }
        self.image = DynamicImage::ImageRgba8(img);
        Ok(self)
    }

     /// Factor > 1.0 increases contrast, < 1.0 decreases it
    pub fn adjust_contrast(&mut self, factor: f32) -> Result<&mut Self, ImageError> {
        let mut img = self.image.to_rgba8();
        for pixel in img.pixels_mut() {
            for c in 0..3 {
                let scaled = (((pixel[c] as f32 / 255.0) - 0.5) * factor + 0.5) * 255.0;
                pixel[c] = scaled.max(0.0).min(255.0) as u8;
            }
        }
        self.image = DynamicImage::ImageRgba8(img);
        Ok(self)
    }

    /// Overlay another image at specified coordinates
    pub fn overlay_image(
        &mut self,
        overlay: &DynamicImage,
        x: u32,
        y: u32,
    ) -> Result<&mut Self, ImageError> {
        if x + overlay.width() > self.image.width() || y + overlay.height() > self.image.height() {
            return Err(ImageError::OperationError(
                "Overlay image exceeds base image bounds".to_string(),
            ));
        }

        // Convert both images to RGBA
        let mut base: ImageBuffer<Rgba<u8>, Vec<u8>> = self.image.to_rgba8();
        let overlay = overlay.to_rgba8();

        // Iterate through overlay pixels and blend them with base image
        for (i, j, pixel) in overlay.enumerate_pixels() {
            let x_pos = x + i;
            let y_pos = y + j;

            if pixel[3] > 0 {  // Only blend non-transparent pixels
                if pixel[3] == 255 {
                    base.put_pixel(x_pos, y_pos, *pixel);
                } else {
                    let base_pixel = base.get_pixel(x_pos, y_pos);
                    let alpha = pixel[3] as f32 / 255.0;
                    let new_pixel = Rgba([
                        ((1.0 - alpha) * base_pixel[0] as f32 + alpha * pixel[0] as f32) as u8,
                        ((1.0 - alpha) * base_pixel[1] as f32 + alpha * pixel[1] as f32) as u8,
                        ((1.0 - alpha) * base_pixel[2] as f32 + alpha * pixel[2] as f32) as u8,
                        255,
                    ]);
                    base.put_pixel(x_pos, y_pos, new_pixel);
                }
            }
        }

        self.image = DynamicImage::ImageRgba8(base);
        Ok(self)
    }

    /// Save the processed image to a file
    pub fn save(&self, path: &str) -> Result<(), ImageError> {
        self.image
            .save(path)
            .map_err(|e| ImageError::OperationError(e.to_string()))
    }

    /// Get the underlying DynamicImage
    pub fn get_image(&self) -> &DynamicImage {
        &self.image
    }
}

// Example usage
fn main() -> Result<(), ImageError> {
    // Load base image
    let mut processor = ImageProcessor::new("base_image.png")?;
    // processor.adjust_contrast(1.5);
    //  processor.blur(1.9);
    // Crop the image
    // processor.crop(100, 100, 500, 500)?;

    // Rotate the image by 45 degrees
    // processor.rotate(45.0)?;

    // Load and overlay another image
    let mut overlay = ImageProcessor::new("overlay_image.png")?;
       overlay.crop(100, 100, 500, 500)?;
    //    overlay.invert();
    //    overlay.rotate(45.0)?;
    processor.overlay_image(overlay.get_image(), 100, 100)?;

    // Save the result
    processor.save("output.png")?;

    Ok(())
}



