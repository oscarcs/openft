use macroquad::{
    prelude::*,
    rand::{gen_range, srand},
};
use plugin_manager::plugin_manager::*;
use std::time::{SystemTime, UNIX_EPOCH};
use texture_manager::texture_manager::*;
use tilemap_manager::tilemap_manager::*;
use util::util::*;

mod plugin_manager;
mod texture_manager;
mod tilemap_manager;
mod util;

const CAMERA_SPEED: f32 = 4.0;
const MAP_SIZE: usize = 200;

#[macroquad::main("OpenFT")]
async fn main() {
    srand(
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            & 0xFFFFFFFFFFFFFFFF) as u64,
    );

    let mut camera: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    let mut map = TileMap::new(MAP_SIZE, MAP_SIZE);

    let mut texture = Texture2D::empty();
    let no_mapping = &ColorMapping {
        target: WHITE,
        channel: ColorMappingChannel::None,
    };
    load_process_texture(&mut texture, "res/GroundSeasonal.png", no_mapping).await;

    for i in 0..4 {
        let tile = DrawableTileData {
            texture: &texture,
            image_data: ImageData::SingleDrawable(Drawable {
                offset: Vec2 { x: 0.0, y: 0.0 },
                origin: Vec2 {
                    x: (TILE_W * i) as f32,
                    y: 0.0,
                },
                width: TILE_W as f32,
                height: TILE_H as f32,
            }),
            size: Tile { x: 1, y: 1, z: 1 },
        };
        map.create_ground_type(tile);
    }

    let plugin_dirs = enumerate_plugins().expect("Plugins not found!");
    let plugins = load_plugins(plugin_dirs);
    let plugin_textures = load_plugin_textures(&plugins).await;
    for plugin in plugins {
        for contribution in plugin.contributions {
            map.create_entity_types(&mut load_drawable_tile_data_from_contribution(
                contribution,
                &plugin.title,
                &plugin_textures,
            ));
        }
    }

    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            if x % 5 == 0 || y % 5 == 0 {
                map.set_ground(x, y, 3);
                continue;
            }

            let base_chance = (x * x) + (y * y);

            let r = gen_range(0, base_chance);
            if r < 100 {
                let t = gen_range(80, map.entity_type_count());
                let entity_info = EntityInfo {
                    height: gen_range(0, 4),
                };
                map.set_entity(x, y, t, Some(entity_info));
            }
            else if r < 200 {
                let t = gen_range(60, 80);
                map.set_entity(x, y, t, None);
            }
        }
    }

    let mut zoom_level: f32 = 2.0;

    let water = Color {
        r: 81.0 / 255.0,
        g: 69.0 / 255.0,
        b: 227.0 / 255.0,
        a: 1.0,
    };

    loop {
        clear_background(water);

        let frame_time = get_frame_time();
        let speed: f32 = CAMERA_SPEED * (60.0 * frame_time);

        if is_key_down(KeyCode::Right) {
            camera.x += speed;
        }
        if is_key_down(KeyCode::Left) {
            camera.x -= speed;
        }
        if is_key_down(KeyCode::Down) {
            camera.y += speed;
        }
        if is_key_down(KeyCode::Up) {
            camera.y -= speed;
        }

        if is_key_pressed(KeyCode::Minus) {
            if zoom_level > 1.0 {
                zoom_level -= 1.0;
            }
        }

        if is_key_pressed(KeyCode::Equal) {
            if zoom_level < 5.0 {
                zoom_level += 1.0;
            }
        }

        let mouse_pos = Vec2 {
            x: mouse_position().0,
            y: mouse_position().1,
        };

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_xy = screen_to_xy(mouse_pos, camera, zoom_level);
            let mouse_iso = xy_to_iso(mouse_xy);

            let x_dest = mouse_iso.x.max(0) as usize;
            let y_dest = mouse_iso.y.max(0) as usize;

            let t = gen_range(0, map.entity_type_count());
            let entity_info = EntityInfo {
                height: gen_range(1, 5),
            };
            let res = map.set_entity(x_dest, y_dest, t, Some(entity_info));

            if !res {
                println!("Couldn't create at {} {}", x_dest, y_dest);
            }
        }

        let screen_xy_origin = screen_to_xy(Vec2 { x: 0.0, y: 0.0 }, camera, zoom_level);
        let screen_xy_extent = screen_to_xy(
            Vec2 {
                x: screen_width(),
                y: screen_height(),
            },
            camera,
            zoom_level,
        );

        let (lower, upper) = min_iso_bounding_box_for_xy((screen_xy_extent, screen_xy_origin));
        let x0 = lower.x.max(0).min(MAP_SIZE as i32) as usize;
        let x1 = upper.x.max(0).min(MAP_SIZE as i32) as usize;
        let y0 = lower.y.max(0).min(MAP_SIZE as i32) as usize;
        let y1 = upper.y.max(0).min(MAP_SIZE as i32) as usize;

        for tx in x0..x1 {
            for ty in y0..y1 {
                let tile = Tile {
                    x: tx as i32,
                    y: ty as i32,
                    z: 0,
                };

                let pos_xy = iso_to_xy(&tile);
                let pos_screen = xy_to_screen(pos_xy, camera, zoom_level);

                match map.get_entity(tx, ty) {
                    Some((entity, drawable, offset)) => {
                        if is_key_down(KeyCode::X) {
                            draw_tile(map.get_ground(tx, ty), pos_screen, MAGENTA, zoom_level);
                            draw_entity(
                                &Some(EntityInfo { height: 0 }),
                                drawable,
                                offset,
                                pos_screen,
                                WHITE,
                                zoom_level,
                            );
                        } else if is_key_down(KeyCode::Z) {
                            draw_tile(map.get_ground(tx, ty), pos_screen, MAGENTA, zoom_level);
                        } else {
                            draw_entity(
                                &entity.entity_info,
                                drawable,
                                offset,
                                pos_screen,
                                WHITE,
                                zoom_level,
                            );
                        }
                    }
                    None => {
                        draw_tile(map.get_ground(tx, ty), pos_screen, WHITE, zoom_level);
                    }
                };
            }
        }

        let str = format!("fps: {:.2}", get_fps());
        draw_text(&str, 10.0, 30.0, 30.0, WHITE);

        if is_key_down(KeyCode::Escape) {
            break;
        }

        next_frame().await
    }
}
