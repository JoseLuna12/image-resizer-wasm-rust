use std::io::{Cursor, Read, Seek};

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
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
    half: Option<DynamicImage>,
    min: Option<DynamicImage>,
    lowest: Option<DynamicImage>,
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
        match extension {
            "jpeg" | "jpg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            _ => {
                panic!("Image Not supported");
            }
        }
    }

    fn load_image_from_buffer(buffer: &[u8], format: &ImageFormat) -> DynamicImage {
        match image::load_from_memory_with_format(buffer, *format) {
            Ok(img) => img,
            Err(error) => {
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
            FilterType::Gaussian,
        );
        log("rceated half image");

        let low_image = image.resize(
            &image_information.width / 4,
            &image_information.height / 4,
            FilterType::Gaussian,
        );
        log("rceated low image");

        let lowest_image = image.resize(
            &image_information.width / 5,
            &image_information.height / 5,
            FilterType::Gaussian,
        );
        log("rceated lowest image");

        ImageResolver {
            image,
            information: image_information,
            half: Some(half_image),
            min: Some(low_image),
            lowest: Some(lowest_image),
        }
    }

    pub fn transform_image_to_unit8(&self, res: ImageResolution) -> Option<Vec<u8>> {
        match res {
            ImageResolution::High => Some(ImageResolver::write_image_to_vec(
                &self.image,
                self.information.extension,
            )),
            ImageResolution::Half => {
                log("transofming half image");
                if let Some(img) = &self.half {
                    Some(ImageResolver::write_image_to_vec(
                        &img,
                        self.information.extension,
                    ))
                } else {
                    None
                }
            }
            ImageResolution::Min => {
                if let Some(img) = &self.min {
                    Some(ImageResolver::write_image_to_vec(
                        &img,
                        self.information.extension,
                    ))
                } else {
                    None
                }
            }
            ImageResolution::Lowest => {
                if let Some(img) = &self.lowest {
                    Some(ImageResolver::write_image_to_vec(
                        &img,
                        self.information.extension,
                    ))
                } else {
                    None
                }
            }
        }
    }
}

#[wasm_bindgen]
pub struct ResultImage {
    image: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub fn process_image(image: &[u8], mime: &str, name: &str, resolution: &str) -> ResultImage {
    let resolution_value = match resolution {
        "high" => ImageResolution::High,
        "half" => ImageResolution::Half,
        "med" => ImageResolution::Min,
        "low" => ImageResolution::Lowest,
        _ => ImageResolution::Half,
    };

    let image_resolver = ImageResolver::new(image, mime, name);

    let result_image = match image_resolver.transform_image_to_unit8(resolution_value) {
        Some(result) => result,
        None => Vec::new(),
    };

    ResultImage {
        image: result_image,
        height: image_resolver.information.width,
        width: image_resolver.information.width,
    }
}
