use image::io::Reader as ImageReader;
use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use std::{fs, path::Path, vec};

const WHITE: Luma<u8> = Luma([255]);
const BLACK: Luma<u8> = Luma([0]);

/// Calculates [Relative Luminance](https://en.wikipedia.org/wiki/Relative_luminance)
/// of an Rgba pixel, which returns a Grayscale value we can work on
///
/// ## Parameters
/// - `pixel`: Rgba pixel
/// ## Returns
/// f32 luminosity
fn luminosity(pixel: &Rgba<u8>) -> f32 {
    let [r, g, b, ..] = pixel.0;
    0.2126 * f32::from(r) + 0.7152 * f32::from(g) + 0.0722 * f32::from(b)
}

/// Checks the pixel at (i + offx, j + offy) on buffer.
/// If it exists, increments its value by `value` and updates buffer in place
///
/// ## Parameters
/// - buffer: Vec<Vec<f32>> of luminosities
/// - i: Initial x
/// - j: Initial y
/// - offx: Offset x
/// - offy: Offset y
/// - value: Value to increment
fn increment_buffer(
    buffer: &mut Vec<Vec<f32>>,
    i: usize,
    j: usize,
    offx: i32,
    offy: i32,
    value: f32,
) {
    let (x, y) = (i as i32 + offx, j as i32 + offy);

    if x < 0 || x > (buffer.len() - 1) as i32 || y < 0 || y > (buffer[0].len() - 1) as i32 {
        return;
    }

    buffer[x as usize][y as usize] += value;
}

/// Uses Atkinson's algorithm to dither the image
///
/// Atkinson error diffusin is as follows
/// ```plaintext
///       | PXL | 1/8 | 1/8 |
/// | 1/8 | 1/8 | 1/8 |
///       | 1/8 |
/// ````
///
/// ## Parameters
/// - `img``: RgbaImage
/// ## Returns
/// GrayImage buffer
fn atkinson(img: &RgbaImage) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut new_img: GrayImage = ImageBuffer::new(w, h);
    let mut buffer: Vec<Vec<f32>> = vec![vec![0.0; h as usize]; w as usize];

    // Fill buffer
    for i in 0..w {
        for j in 0..h {
            buffer[i as usize][j as usize] = luminosity(img.get_pixel(i, j)) / 255.0;
        }
    }

    for x in 0..w {
        for y in 0..h {
            let i = x as usize;
            let j = y as usize;

            let old_pxl = buffer[i][j];
            let new_pxl = if old_pxl > 0.5 { 1.0 } else { 0.0 };
            let error = old_pxl - new_pxl;

            increment_buffer(&mut buffer, i, j, -1, 1, error * 1.0 / 8.0);
            increment_buffer(&mut buffer, i, j, 0, 1, error * 1.0 / 8.0);
            increment_buffer(&mut buffer, i, j, 0, 2, error * 1.0 / 8.0);
            increment_buffer(&mut buffer, i, j, 1, 1, error * 1.0 / 8.0);
            increment_buffer(&mut buffer, i, j, 0, 1, error * 1.0 / 8.0);
            increment_buffer(&mut buffer, i, j, 0, 2, error * 1.0 / 8.0);

            let pxl = if new_pxl == 1.0 { WHITE } else { BLACK };
            new_img.put_pixel(x, y, pxl);
        }
    }

    new_img
}

/// Uses Floyd-Steinberg algorithm to dither the image
///
/// Floyd-Steinberg error diffusin is as follows
/// ```plaintext
///        |  PXL | 7/16 |
/// | 3/16 | 5/16 | 1/16 |
/// ````
///
/// ## Parameters
/// - `img``: RgbaImage
/// ## Returns
/// GrayImage buffer
fn floyd_steinberg(img: &RgbaImage) -> GrayImage {
    let (w, h) = img.dimensions();
    let mut new_img: GrayImage = ImageBuffer::new(w, h);
    let mut buffer: Vec<Vec<f32>> = vec![vec![0.0; h as usize]; w as usize];

    // Fill buffer
    for i in 0..w {
        for j in 0..h {
            buffer[i as usize][j as usize] = luminosity(img.get_pixel(i, j)) / 255.0;
        }
    }

    for x in 0..w {
        for y in 0..h {
            let i = x as usize;
            let j = y as usize;

            let old_pxl = buffer[i][j];
            let new_pxl = if old_pxl > 0.5 { 1.0 } else { 0.0 };
            let error = old_pxl - new_pxl;

            increment_buffer(&mut buffer, i, j, 1, 0, error * 7.0 / 16.0);
            increment_buffer(&mut buffer, i, j, -1, 1, error * 3.0 / 16.0);
            increment_buffer(&mut buffer, i, j, 0, 1, error * 5.0 / 16.0);
            increment_buffer(&mut buffer, i, j, 1, 1, error * 1.0 / 16.0);

            let pxl = if new_pxl == 1.0 { WHITE } else { BLACK };
            new_img.put_pixel(x, y, pxl);
        }
    }

    new_img
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: ./dithering /path/to/image");
        return;
    }

    let file_path = Path::new(&args[1]);

    let img = ImageReader::open(file_path)
        .expect(format!("failed to open {}", file_path.to_string_lossy()).as_str())
        .decode()
        .expect("failed to decode")
        .to_rgba8();

    let atkinson_dither = atkinson(&img);
    let floyd_dither = floyd_steinberg(&img);

    if let Err(e) = fs::create_dir_all("./out") {
        eprintln!("Error creating the output folder, {:?}", e);
        return;
    }

    let file_name = file_path.file_stem().unwrap().to_string_lossy();
    let file_ext = file_path.extension().unwrap().to_string_lossy();

    atkinson_dither
        .save(Path::new(&format!(
            "./out/{}.atkinson.{}",
            file_name, file_ext
        )))
        .expect("failed to save");

    floyd_dither
        .save(Path::new(&format!(
            "./out/{}.floyd.{}",
            file_name, file_ext
        )))
        .expect("failed to save");
}
