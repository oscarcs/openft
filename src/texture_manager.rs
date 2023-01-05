pub mod texture_manager {
    use crate::{
        plugin_manager::plugin_manager::{
            ColorMapping, ColorMappingChannel, Contribution, ContributionImageData,
            ContributionSprite, Plugin,
        },
        tilemap_manager::tilemap_manager::{EntityInfo, Tile},
        util::util::{min_xy_bounding_box_for_iso_size, TILE_H_HALF, TILE_W, TILE_W_HALF},
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

    #[derive(Debug)]
    pub struct DrawableTileData<'a> {
        pub texture: &'a Texture2D,
        pub image_data: ImageData,
        pub size: Tile,
    }

    #[derive(Debug)]
    pub enum ImageData {
        SingleDrawable(Drawable),
        MultistoreyDrawable(Drawable, Drawable, Drawable),
    }

    #[derive(Debug, Clone, Copy)]
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
        let (w, h) = min_xy_bounding_box_for_iso_size(contribution.size.x, contribution.size.y);

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

            for id in &contribution.image_data {
                let mut flip = false;

                let image_data = match id {
                    ContributionImageData::ContributionSprite(s) => {
                        if s.opposite {
                            flip = true;
                        }
                        ImageData::SingleDrawable(contribution_sprite_to_drawable(s, w, h))
                    }
                    ContributionImageData::ContributionMultistorey(s) => {
                        if s.top.opposite || s.middle.opposite || s.bottom.opposite {
                            flip = true;
                        }
                        ImageData::MultistoreyDrawable(
                            contribution_sprite_to_drawable(&s.top, w, h),
                            contribution_sprite_to_drawable(&s.middle, w, h),
                            contribution_sprite_to_drawable(&s.bottom, w, h),
                        )
                    }
                    ContributionImageData::ContributionAutotile(_, _) => todo!(),
                };

                let size = match flip {
                    true => Tile {
                        x: contribution.size.y,
                        y: contribution.size.x,
                        z: contribution.size.z,
                    },
                    false => contribution.size,
                };

                if size.x != size.y {}

                drawables.push(DrawableTileData {
                    texture,
                    image_data,
                    size,
                });
            }
        }
        drawables
    }

    fn contribution_sprite_to_drawable(s: &ContributionSprite, w: i32, h: i32) -> Drawable {
        Drawable {
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
        }
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

        // let (r, g, b) = (mapping.target.r, mapping.target.g, mapping.target.b);
        let target: Color = mapping.target;

        // if r > g && r > b {
        //     target = RED;
        // }
        // else if g > r && g > b {
        //     target = GREEN;
        // }
        // else if b > r && b > g {
        //     target = BLUE;
        // }
        // else {
        //     target = WHITE;
        // }

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
                    r: target.r * brightness,
                    g: target.g * brightness,
                    b: target.b * brightness,
                    a: 1.0,
                };

                image.set_pixel(x as u32, y as u32, color);
            }
        }
    }

    pub fn draw_entity(
        entity_info: &Option<EntityInfo>,
        tile: &DrawableTileData,
        tile_offset: Tile,
        destination: Vec2,
        color: Color,
        scale: f32,
    ) {
        match &tile.image_data {
            ImageData::SingleDrawable(image) => {
                let mut drawable = *image;

                if tile.size.y > 1 {
                    drawable.origin.x = drawable.origin.x + (TILE_W_HALF * tile_offset.y) as f32;
                    drawable.offset.y = drawable.offset.y - (TILE_H_HALF * tile_offset.y) as f32;
                    drawable.width = TILE_W as f32;
                } else if tile.size.x > 1 {
                    drawable.origin.x = drawable.origin.x + (TILE_W_HALF * tile_offset.x) as f32;
                    drawable.offset.y = drawable.offset.y + (TILE_H_HALF * tile_offset.x) as f32;
                    drawable.width = TILE_W as f32;
                }

                draw(&drawable, &tile.texture, destination, color, scale);
            }
            ImageData::MultistoreyDrawable(top, middle, bottom) => {
                let h = match entity_info {
                    Some(i) => i.height,
                    None => 1,
                };

                draw(&bottom, &tile.texture, destination, color, scale);

                let mut y = destination.y;
                for _ in 1..=h {
                    y -= scale * (middle.height - middle.offset.y);
                    let dest = Vec2 {
                        x: destination.x,
                        y,
                    };
                    draw(&middle, &tile.texture, dest, color, scale);
                }

                y -= scale * (top.height - top.offset.y);
                let dest = Vec2 {
                    x: destination.x,
                    y,
                };
                draw(&top, &tile.texture, dest, color, scale);
            }
        }
    }

    pub fn draw_tile(tile: &DrawableTileData, destination: Vec2, color: Color, scale: f32) {
        match &tile.image_data {
            ImageData::SingleDrawable(image) => {
                draw(&image, &tile.texture, destination, color, scale);
            }
            ImageData::MultistoreyDrawable(_, _, _) => panic!("A tile cannot be multistorey!"),
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
