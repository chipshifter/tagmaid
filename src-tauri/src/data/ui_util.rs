use image::io::Reader as ImageReader;
use std::path::PathBuf;

/// Creates a jpeg thumbnail with a given size and path. Saves the thumbnail in the same
/// location as the original fine and returns the path
/// (of the form /path/to/orig/file/thumb_\[original_filename\].jpg)
pub fn create_image_thumbnail(path: &PathBuf, max_width: u32, max_height: u32) -> PathBuf {
    if !&path.exists() {
        return PathBuf::new();
    }
    let file_name = path
        .as_path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut path_thumbnail = path.clone();
    path_thumbnail.pop();
    path_thumbnail.push(String::from("thumb_") + &file_name);
    path_thumbnail.set_extension("jpg");
    if !path_thumbnail.exists() {
        let load_image = ImageReader::open(&path).unwrap().decode();
        match load_image {
            Ok(img) => {
                let img_thumbnail = img.thumbnail(max_width, max_height);
                img_thumbnail
                    .save_with_format(&path_thumbnail, image::ImageFormat::Jpeg)
                    .ok();
            }
            // This is dumb but it wok for us
            Err(_) => {
                return PathBuf::new();
            }
        }
    }

    return path_thumbnail;
}
