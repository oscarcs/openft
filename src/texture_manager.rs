pub mod texture_manager {
    use crate::{min_xy_bounding_box_for_iso_size, plugin_manager::plugin_manager::Plugin};
    use macroquad::prelude::*;
    use std::collections::HashMap;

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
        pub height: f32,
    }

    pub async fn load_plugin_textures(plugins: &Vec<Plugin>) -> HashMap<String, Texture2D> {
        let mut plugin_textures = HashMap::<String, Texture2D>::new();
        for plugin in plugins {
            for contribution in &plugin.contributions {
                for sprite in &contribution.sprites {
                    let mut texture_path = plugin.filename.clone();
                    texture_path.push(sprite.picture_ref.as_str());

                    // Load the texture into GPU memory if it isn't already
                    let texture_key = texture_path.to_str().unwrap();
                    if !plugin_textures.contains_key(texture_key) {
                        let mut texture = Texture2D::empty();
                        load_process_texture(&mut texture, texture_key).await;
                        plugin_textures.insert(texture_key.to_owned(), texture);
                    }
                }
            }
        }
        plugin_textures
    }

    pub fn load_drawables_from_plugins<'a>(
        drawables: &mut Vec<Drawable<'a>>,
        plugins: &Vec<Plugin>,
        plugin_textures: &'a HashMap<String, Texture2D>,
    ) {
        for plugin in plugins {
            for contribution in &plugin.contributions {
                let (w, h) =
                    min_xy_bounding_box_for_iso_size(contribution.size_x, contribution.size_y);

                for sprite in &contribution.sprites {
                    let mut texture_path = plugin.filename.clone();
                    texture_path.push(sprite.picture_ref.as_str());

                    let texture_key = texture_path.to_str().unwrap();

                    let texture = match plugin_textures.get(texture_key) {
                        Some(texture) => texture,
                        None => {
                            println!("Warning: couldn't retrieve texture '{}' while loading Drawable", texture_key);
                            continue;
                        } 
                    };

                    drawables.push(Drawable {
                        height: (h + sprite.offset) as f32,
                        width: w as f32,
                        offset: Vec2 {
                            x: 0.0,
                            y: sprite.offset as f32,
                        },
                        origin: Vec2 {
                            x: sprite.origin_x as f32,
                            y: sprite.origin_y as f32,
                        },
                        texture,
                    });
                }
            }
        }
    }

    pub async fn load_process_texture(texture: &mut Texture2D, filename: &str) {
        let mut image = if filename.contains(".bmp") || filename.contains(".BMP") {
            let bmp = bmp::open(filename).unwrap();
            let w = bmp.get_width();
            let h = bmp.get_height();

            let mut gen_image = Image::gen_image_color(w as u16, h as u16, TRANSPARENT);
            for x in 0..w {
                for y in 0..h {
                    let p = bmp.get_pixel(x, y);
                    gen_image.set_pixel(
                        x,
                        y,
                        Color {
                            r: p.r as f32 / 255.0,
                            g: p.g as f32 / 255.0,
                            b: p.b as f32 / 255.0,
                            a: 1.0,
                        },
                    );
                }
            }
            gen_image
        } else {
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
