use std::io::{Cursor, Read, Seek};

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

enum ImageResolution {
    High,
    Half,
    Min,
    Lowest,
}

struct ImageInformation {
    name: String,
    width: u32,
    height: u32,
    extension: ImageFormat,
}

struct ImageResolver {
    image: DynamicImage,
    information: ImageInformation,
    half: DynamicImage,
    min: DynamicImage,
    lowest: DynamicImage,
}

trait ImageHelpers {
    fn get_extension(extesnsion: &str) -> ImageFormat;
    fn load_image_from_buffer(buffer: &[u8], format: &ImageFormat) -> DynamicImage;
    fn get_information_from_dyn_image(
        image: &DynamicImage,
        extension: ImageFormat,
        name: &str,
    ) -> ImageInformation;

    fn write_image_to_vec(image: &DynamicImage, extension: ImageFormat) -> Vec<u8>;
}

impl ImageHelpers for ImageResolver {
    fn get_extension(extension: &str) -> ImageFormat {
        log(&format!("looging for extension: {}", extension));
        match extension {
            "jpeg" | "jpg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "gif" => ImageFormat::Gif,
            "webp" => ImageFormat::WebP,
            _ => {
                log(&format!("image extension not suported: {}", extension));
                panic!("Image Not supported");
            }
        }
    }

    fn load_image_from_buffer(buffer: &[u8], format: &ImageFormat) -> DynamicImage {
        match image::load_from_memory_with_format(buffer, *format) {
            Ok(img) => img,
            Err(error) => {
                log(&format!("could not open the file {:?}", error));
                panic!("There was a problem opening the file: {:?}", error)
            }
        }
    }

    fn get_information_from_dyn_image(
        image: &DynamicImage,
        extension: ImageFormat,
        name: &str,
    ) -> ImageInformation {
        let (width, height) = image.dimensions();
        ImageInformation {
            name: String::from(name),
            width,
            height,
            extension,
        }
    }

    fn write_image_to_vec(image: &DynamicImage, extension: ImageFormat) -> Vec<u8> {
        let mut memory_cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        log("creating emtpy cursor");

        match image.write_to(&mut memory_cursor, extension) {
            Ok(value) => value,
            Err(err) => {
                log(&format!(
                    "There was a problem writing the resulting buffer {}",
                    err
                ));
                panic!(
                    "There was a problem writing the resulting buffer: {:?}",
                    err
                )
            }
        };

        log("creating emtpy cursor");

        memory_cursor.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut out: Vec<u8> = Vec::new();
        memory_cursor.read_to_end(&mut out).unwrap();
        out
    }
}

impl ImageResolver {
    pub fn new(buffer: &[u8], extension: &str, name: &str) -> Self {
        log("creating image");
        log(extension);
        log(name);
        let extension_format = ImageResolver::get_extension(extension);

        let image = ImageResolver::load_image_from_buffer(buffer, &extension_format);

        let image_information =
            ImageResolver::get_information_from_dyn_image(&image, extension_format, name);

        let half_image = image.resize(
            &image_information.width / 2,
            &image_information.height / 2,
            FilterType::Nearest,
        );
        let half_dimentions = half_image.dimensions();
        log(&format!(
            "widht: {} height: {}",
            half_dimentions.0, half_dimentions.1
        ));
        log("rceated half image");

        let low_image = image.resize(
            &image_information.width / 4,
            &image_information.height / 4,
            FilterType::Nearest,
        );
        let low_dimentions = low_image.dimensions();
        log(&format!(
            "widht: {} height: {}",
            low_dimentions.0, low_dimentions.1
        ));
        log("rceated low image");

        let lowest_image = image.resize(
            &image_information.width / 5,
            &image_information.height / 5,
            FilterType::Nearest,
        );
        let lowest_dimentions = lowest_image.dimensions();
        log(&format!(
            "widht: {} height: {}",
            lowest_dimentions.0, lowest_dimentions.1
        ));
        log("rceated lowest image");

        ImageResolver {
            information: image_information,
            image,
            half: half_image,
            min: low_image,
            lowest: lowest_image,
        }
    }

    pub fn update_dimensions(&mut self, resolution: &ImageResolution) {
        let image = match resolution {
            ImageResolution::High => &self.image,
            ImageResolution::Half => &self.half,
            ImageResolution::Min => &self.min,
            ImageResolution::Lowest => &self.lowest,
        };

        let (width, height) = image.dimensions();
        self.information.height = height;
        self.information.width = width;
    }

    pub fn transform_image_to_unit8(&mut self, res: ImageResolution) -> Vec<u8> {
        self.update_dimensions(&res);
        match res {
            ImageResolution::High => {
                ImageResolver::write_image_to_vec(&self.image, self.information.extension)
            }
            ImageResolution::Half => {
                ImageResolver::write_image_to_vec(&self.half, self.information.extension)
            }
            ImageResolution::Min => {
                ImageResolver::write_image_to_vec(&self.min, self.information.extension)
            }
            ImageResolution::Lowest => {
                ImageResolver::write_image_to_vec(&self.lowest, self.information.extension)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResultImage {
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[wasm_bindgen]
pub fn process_image(image: &[u8], mime: &str, name: &str, resolution: &str) -> JsValue {
    let resolution_value = match resolution {
        "high" => ImageResolution::High,
        "half" => ImageResolution::Half,
        "med" => ImageResolution::Min,
        "low" => ImageResolution::Lowest,
        _ => {
            log(&format!("default value activated: {}", resolution));
            ImageResolution::Half
        }
    };

    let mut image_resolver = ImageResolver::new(image, mime, name);

    let vector_image = image_resolver.transform_image_to_unit8(resolution_value);
    let result_image = ResultImage {
        image: vector_image,
        height: image_resolver.information.width,
        width: image_resolver.information.width,
    };

    serde_wasm_bindgen::to_value(&result_image).unwrap()
}
