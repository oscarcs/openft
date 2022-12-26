pub mod texture_manager {
    use macroquad::prelude::*;

    const TRANSPARENT_COLOR: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    const TRANSPARENT: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    };

    pub struct Drawable<'a> {
        pub texture: &'a Texture2D,
        pub offset: Vec2,
        pub origin: Vec2,
        pub width: f32,
        pub height: f32
    }

    pub async fn load_process_texture(texture: &mut Texture2D, filename: &str) {


        let mut image = if filename.contains(".bmp") {
            let bmp = bmp::open(filename).unwrap();
            let w = bmp.get_width();
            let h = bmp.get_height();

            let mut i = Image::gen_image_color(w as u16, h as u16, TRANSPARENT);
            for x in 0..w {
                for y in 0..h {
                    let p = bmp.get_pixel(x, y);
                    i.set_pixel(x, y, Color {
                        r: p.r as f32 / 255.0,
                        g: p.g as f32 / 255.0,
                        b: p.b as f32 / 255.0,
                        a: 1.0
                    });
                }
            }
            i
        }
        else {
            load_image(filename).await.unwrap()
        };
        make_transparent(&mut image);

        *texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
    }

    pub fn make_transparent(image: &mut Image) {
        let w = image.width();
        let h = image.height();

        for x in 0..w {
            for y in 0..h {
                let image_pixel = image.get_pixel(x as u32, y as u32);
                if image_pixel.eq(&TRANSPARENT_COLOR) {
                    image.set_pixel(x as u32, y as u32, TRANSPARENT);
                }
            }
        }
    }

    pub fn draw(drawable: &Drawable, destination: Vec2, color: Color, scale: f32) {
        let params = DrawTextureParams {
            dest_size: Some(Vec2 {
                x: drawable.width * scale,
                y: drawable.height * scale,
            }),
            source: Some(Rect {
                x: drawable.origin.x,
                y: drawable.origin.y,
                w: drawable.width,
                h: drawable.height,
            }),
            ..Default::default()
        };

        draw_texture_ex(
            *drawable.texture,
            destination.x - (drawable.offset.x * scale),
            destination.y - (drawable.offset.y * scale),
            color,
            params,
        );
    }
}
