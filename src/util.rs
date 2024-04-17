
use image::GenericImageView;
use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout}; // Using image crate: https://github.com/image-rs/image
use webp::{Encoder, WebPMemory}; // Using webp crate: https://github.com/jaredforth/webp
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::fs;

fn img_to_png(file_path: &String) -> Option<String> {
    // Use the open function to load an image from a Path.
// `open` returns a `DynamicImage` on success.
let img = image::open(file_path).unwrap();

// The dimensions method returns the images width and height.
println!("dimensions {:?}", img.dimensions());
let path: &Path = Path::new(file_path);
let parent_directory: &Path = path.parent().unwrap();
let filename_original_image = path.file_stem().unwrap().to_str().unwrap();
// The color method returns the image's `ColorType`.
println!("{:?}", img.color());
let png_folder_path = format!("{}/png", parent_directory.to_str().unwrap());
let png_image_path = format!(
    "{}/{}.png",
    png_folder_path.to_string(),
    filename_original_image
);
let _ =fs::create_dir_all(png_folder_path.to_string());
// Write the contents of this image to the Writer in PNG format.
img.save(png_image_path.to_string()).unwrap();

return Some(png_image_path.to_string());
}

/*
Function which converts an image in PNG or JPEG format to WEBP.
:param file_path: &String with the path to the image to convert.
:return Option<String>: Return the path of the WEBP-image as String when succesfull, returns None if function fails.
*/
fn image_to_webp(file_path: &String) -> Option<String> {
// Open path as DynamicImage
//let image: DynamicImage = ImageReader::open(file_path).unwrap().decode().unwrap();
let image = ImageReader::open(file_path);
let image: DynamicImage = match image {
    Ok(img) => img.with_guessed_format().unwrap().decode().unwrap(), //ImageReader::with_guessed_format() function guesses if image needs to be opened in JPEG or PNG format.
    Err(e) => {
        println!("Error: {}", e);
        return None;
    }
};

// Make webp::Encoder from DynamicImage.
let encoder: Encoder = Encoder::from_image(&image).unwrap();

// Encode image into WebPMemory.
let encoded_webp: WebPMemory = encoder.encode(65f32);

// Put webp-image in a separate webp-folder in the location of the original image.
let path: &Path = Path::new(file_path);
let parent_directory: &Path = path.parent().unwrap();
let webp_folder_path = format!("{}/webp", parent_directory.to_str().unwrap());
match std::fs::create_dir_all(webp_folder_path.to_string()) {
    Ok(_) => {}
    Err(e) => {
        println!("Error {}", e);
        return None;
    }
}

// Get filename of original image.
let filename_original_image = path.file_stem().unwrap().to_str().unwrap();

// Make full output path for webp-image.
let webp_image_path = format!(
    "{}/{}.webp",
    webp_folder_path.to_string(),
    filename_original_image
);

// Make File-stream for WebP-result and write bytes into it, and save to path "output.webp".
let mut webp_image = File::create(webp_image_path.to_string()).unwrap();
match webp_image.write_all(encoded_webp.as_bytes()) {
    Err(e) => {
        println!("Error: {}", e);
        return None;
    }
    Ok(_) => return Some(webp_image_path),
}
}