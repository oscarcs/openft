pub mod texture_manager {
    use crate::{
        min_xy_bounding_box_for_iso_size,
        plugin_manager::plugin_manager::{
            ColorMapping, ColorMappingChannel, Contribution, ContributionImageData, Plugin,
        },
    };
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

    pub struct DrawableTileData<'a> {
        pub texture: &'a Texture2D,
        pub image_data: ImageData,
    }

    pub enum ImageData {
        SingleDrawable(Drawable),
    }

    pub struct Drawable {
        pub offset: Vec2,
        pub origin: Vec2,
        pub width: f32,
        pub height: f32,
    }

    pub async fn load_plugin_textures(plugins: &Vec<Plugin>) -> HashMap<String, Texture2D> {
        let mut plugin_textures = HashMap::<String, Texture2D>::new();
        for plugin in plugins {
            for contribution in &plugin.contributions {
                let mut texture_path = plugin.filename.clone();
                texture_path.push(contribution.image_ref.as_str());
                let texture_full_path = texture_path.to_str().unwrap();

                let key_base = format!("{}-{}", plugin.title, contribution.image_ref);

                for (i, mapping) in contribution.color_mappings.iter().enumerate() {
                    let key = format!("{}-{}", key_base, i);

                    // Load the texture into GPU memory if it isn't already
                    if !plugin_textures.contains_key(&key) {
                        let mut texture = Texture2D::empty();
                        load_process_texture(&mut texture, texture_full_path, mapping).await;
                        plugin_textures.insert(key.to_owned(), texture);
                    }
                }
            }
        }
        plugin_textures
    }

    pub fn load_drawable_tile_data_from_contribution<'a>(
        contribution: Contribution,
        title: &str,
        textures: &'a HashMap<String, Texture2D>,
    ) -> Vec<DrawableTileData<'a>> {
        let (w, h) = min_xy_bounding_box_for_iso_size(contribution.x, contribution.y);

        let key_base = format!("{}-{}", title, contribution.image_ref);
        let mut drawables = Vec::new();

        for i in 0..contribution.color_mappings.len() {
            let key = format!("{}-{}", key_base, i);

            let texture = match textures.get(&key) {
                Some(texture) => texture,
                None => panic!(
                    "Warning: couldn't retrieve texture '{}' while loading tile data",
                    key
                ),
            };

            let image_data = match contribution.image_data.len() {
                0 => panic!("No image data found!"),
                1 => &contribution.image_data[0],
                x => {
                    println!("Found {} sets of image data for contribution, using only the first one for now.", x);
                    &contribution.image_data[0]
                }
            };

            let drawable = match image_data {
                ContributionImageData::ContributionSprite(s) => Drawable {
                    offset: Vec2 {
                        x: 0.0,
                        y: s.offset as f32,
                    },
                    origin: Vec2 {
                        x: s.origin_x as f32,
                        y: s.origin_y as f32,
                    },
                    width: w as f32,
                    height: (h + s.offset) as f32,
                },
                ContributionImageData::ContributionPictures(_) => {
                    todo!("Multi-part image data not supported yet");
                }
            };

            let image_data = ImageData::SingleDrawable(drawable);

            drawables.push(DrawableTileData {
                texture,
                image_data,
            });
        }
        drawables
    }

    pub async fn load_process_texture(
        texture: &mut Texture2D,
        filename: &str,
        mapping: &ColorMapping,
    ) {
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

        map_colors(&mut image, mapping);
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

    pub fn map_colors(image: &mut Image, mapping: &ColorMapping) {
        let w = image.width();
        let h = image.height();

        for x in 0..w {
            for y in 0..h {
                let image_pixel = image.get_pixel(x as u32, y as u32);

                let brightness: f32;
                match mapping.channel {
                    ColorMappingChannel::None => continue,
                    ColorMappingChannel::Red => {
                        if image_pixel.b != 0.0 || image_pixel.g != 0.0 {
                            continue;
                        }
                        brightness = image_pixel.r;
                    }
                    ColorMappingChannel::Green => {
                        if image_pixel.b != 0.0 || image_pixel.g != 0.0 {
                            continue;
                        }
                        brightness = image_pixel.g;
                    }
                    ColorMappingChannel::Blue => {
                        if image_pixel.b != 0.0 || image_pixel.g != 0.0 {
                            continue;
                        }
                        brightness = image_pixel.b;
                    }
                }

                let color = Color {
                    r: mapping.target.r * (brightness / 1.0),
                    g: mapping.target.g * (brightness / 1.0),
                    b: mapping.target.b * (brightness / 1.0),
                    a: 1.0,
                };

                image.set_pixel(x as u32, y as u32, color);
            }
        }
    }

    pub fn draw_tile(tile: &DrawableTileData, destination: Vec2, color: Color, scale: f32) {
        match &tile.image_data {
            ImageData::SingleDrawable(image) => {
                draw(&image, &tile.texture, destination, color, scale);
            }
        }
    }

    pub fn draw(
        drawable: &Drawable,
        texture: &Texture2D,
        destination: Vec2,
        color: Color,
        scale: f32,
    ) {
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
            *texture,
            destination.x - (drawable.offset.x * scale),
            destination.y - (drawable.offset.y * scale),
            color,
            params,
        );
    }
}
