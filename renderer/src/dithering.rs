use image::{Rgb, Luma, DynamicImage, ImageBuffer};
use image::imageops::colorops;

struct SimpleColorMap;

impl colorops::ColorMap for SimpleColorMap {
    type Color = Luma<u8>;

    fn index_of(&self, pixel: &Self::Color) -> usize {
        if pixel[0] < 128 {
            0
        } else {
            1
        }
    }

    fn map_color(&self, pixel: &mut Self::Color) {
        let index = self.index_of(pixel);
        if index == 0 {
            pixel.0 = [0];  // Fully black
        } else {
            pixel.0 = [255];  // Fully white
        }
    }
}

pub fn dither_image_grayscale(image: DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // Convert the image to grayscale
    let mut grayscale_img = image.to_luma8();

    // Perform dithering
    colorops::dither(&mut grayscale_img, &SimpleColorMap);

	let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(grayscale_img.width(), grayscale_img.height(), |x, y| {
		let pixel = grayscale_img.get_pixel(x, y);
		Rgb([pixel[0], pixel[0], pixel[0]])
	});

	rgb_img
}