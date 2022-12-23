pub mod texture_manager {
    use macroquad::{prelude::*};

    const TRANSPARENT_COLOR: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0
    };

    const TRANSPARENT: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0
    };

    pub async fn load_process_textures(filenames: &[&str]) -> Vec<Texture2D> {
        let mut house_images = Vec::<Image>::new();
        for name in filenames {
            house_images.push(load_image(name).await.unwrap());
        }
        
        for mut house_image in house_images.iter_mut() {
            make_transparent(&mut house_image);
        }
    
        let mut house_textures = Vec::new();
        for house_image in house_images {
            let house_texture: Texture2D = Texture2D::from_image(&house_image);
            house_texture.set_filter(FilterMode::Nearest);
            house_textures.push(house_texture);
        }

        house_textures
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

    pub fn draw(texture: &Texture2D, origin: Vec2, tile: u32, color: Color, scale: f32) {
        let sprite_w = 32.0;//32.0 + offset.x;
        let sprite_h = texture.height();//16.0 + offset.y;
        let offset = Vec2 { x: 0.0, y: sprite_h - 16.0 };
        
        let params = DrawTextureParams {
            dest_size: Some(Vec2 {
                x: sprite_w * scale,
                y: sprite_h * scale,
            }),
            source: Some(Rect {
                x: 32.0 * (tile as f32),
                y: 0.0,
                w: sprite_w,
                h: sprite_h
            }),
            ..Default::default()
        };

        draw_texture_ex(
            *texture,
            origin.x - (offset.x * scale),
            origin.y - (offset.y * scale),
            color,
            params);
    }


}