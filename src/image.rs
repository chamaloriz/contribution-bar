use image::{ImageBuffer, Rgba, RgbaImage};

const IMAGE_HEIGHT: u32 = 32;
const IMAGE_WIDTH: u32 = 254;

fn get_color(name: usize) -> Rgba<u8> {
    match name {
        1 => Rgba([6, 58, 23, 255]),    // #063A17 1-5
        2 => Rgba([25, 108, 46, 255]),  // #196C2E 5-10
        3 => Rgba([46, 169, 67, 255]),  // #2EA043 10-15
        4 => Rgba([86, 211, 100, 255]), // #56D364 15 +
        _ => Rgba([21, 28, 35, 255]),   // #151C23 0
    }
}

pub fn generate_image() -> RgbaImage {
    let square_size = 32;
    let spacing = 5;

    let mut img: RgbaImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    for i in 0..7 {
        let color = get_color(i);

        let x_start = i * (square_size + spacing);
        for y in 0..square_size {
            for x in 0..square_size {
                if x_start + x < IMAGE_WIDTH as usize && y < IMAGE_HEIGHT as usize {
                    img.put_pixel((x_start + x) as u32, y as u32, color);
                }
            }
        }
    }

    return img;
}

pub fn generate_icon() -> tray_icon::Icon {
    let img = generate_image();
    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, IMAGE_WIDTH, IMAGE_HEIGHT).expect("Failed to create icon")
}

#[test]
pub fn generate_icon_file() {
    let img = generate_image();
    img.save("icon-test.png").expect("Failed to save image");
}
