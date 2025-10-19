use image::{ImageBuffer, Rgba, RgbaImage};

pub const NUMBER_OF_SQUARES: usize = 7;
const SQUARE_SIZE: usize = 31;
const SPACING: usize = 5;
const IMAGE_HEIGHT: u32 = 31;
const IMAGE_WIDTH: u32 = (((SQUARE_SIZE + SPACING) * NUMBER_OF_SQUARES) - SPACING) as u32;

fn get_color(name: u8) -> Rgba<u8> {
    match name {
        1 => Rgba([6, 58, 23, 255]),    // #063A17
        2 => Rgba([25, 108, 46, 255]),  // #196C2E
        3 => Rgba([46, 169, 67, 255]),  // #2EA043
        4 => Rgba([86, 211, 100, 255]), // #56D364
        _ => Rgba([21, 28, 35, 255]),   // #151C23
    }
}

pub fn generate_image(contributions: Vec<u8>) -> RgbaImage {
    let mut img: RgbaImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    for (i, _value) in contributions.iter().enumerate().take(NUMBER_OF_SQUARES) {
        let color = get_color(contributions[i]);
        let x_start = i * (SQUARE_SIZE + SPACING);

        let square_center = ((x_start + SQUARE_SIZE / 2) as u32, (SQUARE_SIZE / 2) as u32);

        for pixel_y in 0..SQUARE_SIZE {
            for x in 0..SQUARE_SIZE {
                let pixel_x = x_start + x;

                let distance_to_center = ((pixel_x as f64 - square_center.0 as f64).powi(2)
                    + (pixel_y as f64 - square_center.1 as f64).powi(2))
                .sqrt();

                if distance_to_center >= 18.0 {
                    // round the corner
                    continue;
                }

                if x_start + x < IMAGE_WIDTH as usize && pixel_y < IMAGE_HEIGHT as usize {
                    img.put_pixel(pixel_x as u32, pixel_y as u32, color);
                }
            }
        }
    }

    img
}

pub fn generate_icon(contributions: Vec<u8>) -> tray_icon::Icon {
    let img = generate_image(contributions);
    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, IMAGE_WIDTH, IMAGE_HEIGHT).expect("Failed to create icon")
}

#[test]
pub fn generate_icon_file() {
    let testing_vec: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6];
    let img = generate_image(testing_vec);
    img.save("icon-test.png").expect("Failed to save image");
}
