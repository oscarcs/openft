use macroquad::prelude::*;
use plugin_manager::plugin_manager::*;
use texture_manager::texture_manager::*;

mod plugin_manager;
mod texture_manager;

const TILE_W: i32 = 32;
const TILE_H: i32 = 16;
const TILE_W_HALF: i32 = TILE_W / 2;
const TILE_H_HALF: i32 = TILE_H / 2;

const CAMERA_SPEED: f32 = 4.0;
const MAP_SIZE: usize = 200;

#[macroquad::main("OpenFT")]
async fn main() {
    let mut tile_data = Vec::<DrawableTileData>::new();

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
        };
        tile_data.push(tile);
    }

    let plugin_dirs = enumerate_plugins().expect("Plugins not found!");
    let plugins = load_plugins(plugin_dirs);
    let plugin_textures = load_plugin_textures(&plugins).await;
    for plugin in plugins {
        for contribution in plugin.contributions {
            tile_data.append(&mut load_drawable_tile_data_from_contribution(
                contribution,
                &plugin.title,
                &plugin_textures,
            ));
        }
    }

    let mut zoom_level: f32 = 2.0;

    let water = Color {
        r: 81.0 / 255.0,
        g: 69.0 / 255.0,
        b: 227.0 / 255.0,
        a: 1.0,
    };

    let mut camera: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    let mut map: [[u32; MAP_SIZE]; MAP_SIZE] = [[0; MAP_SIZE]; MAP_SIZE];

    for x in 0..map.len() {
        for y in 0..map[0].len() {
            map[x][y] = 0;
        }
    }

    let mut selected_tiles = Vec::<Tile>::new();

    loop {
        let mut calls = 0;

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
        let mouse_xy = screen_to_xy(mouse_pos, camera, zoom_level);
        let mouse_iso = xy_to_iso(mouse_xy);

        if is_mouse_button_down(MouseButton::Left) {
            if !selected_tiles.contains(&mouse_iso) {
                selected_tiles.push(mouse_iso);
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            selected_tiles.clear();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_xy = screen_to_xy(mouse_pos, camera, zoom_level);
            let mouse_iso = xy_to_iso(mouse_xy);
            let new_value = rand::gen_range(4, tile_data.len());
            map[mouse_iso.x.max(0) as usize][mouse_iso.y.max(0) as usize] = new_value as u32;
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
        let x0 = lower.x.max(0) as usize;
        let x1 = upper.x.max(0) as usize;
        let y0 = lower.y.max(0) as usize;
        let y1 = upper.y.max(0) as usize;

        for tx in x0..x1 {
            for ty in y0..y1 {
                let tile = Tile {
                    x: tx as i32,
                    y: ty as i32,
                    z: 0,
                };

                let pos_xy = iso_to_xy(&tile);
                let pos_screen = xy_to_screen(pos_xy, camera, zoom_level);
                let color = if !selected_tiles.contains(&tile) {
                    WHITE
                } else {
                    PINK
                };

                let val = map[tx][ty];
                let data = &tile_data[val as usize];
                draw_tile(data, pos_screen, color, zoom_level);

                calls += 1;
            }
        }

        let str = format!("calls: {} / fps: {:.2}", calls, get_fps());
        draw_text(&str, 20.0, 20.0, 30.0, RED);

        if is_key_down(KeyCode::Escape) {
            break;
        }

        next_frame().await
    }
}

struct Tile {
    x: i32,
    y: i32,
    z: i32,
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

fn iso_to_xy(tile: &Tile) -> Vec2 {
    Vec2 {
        x: ((tile.x - tile.y - 1) * TILE_W_HALF) as f32,
        y: ((tile.x + tile.y) * TILE_H_HALF) as f32,
    }
}

// fn iso_to_xy_midpoint(tile: &Tile) -> Vec2 {
//     Vec2 {
//         x: ((tile.x - tile.y) * TILE_W_HALF) as f32,
//         y: ((tile.x + tile.y + 1) * TILE_H_HALF) as f32
//     }
// }

fn xy_to_screen(point: Vec2, origin: Vec2, scale: f32) -> Vec2 {
    Vec2 {
        x: (point.x - origin.x) * scale,
        y: (point.y - origin.y) * scale,
    }
}

fn xy_to_iso(point: Vec2) -> Tile {
    let px = point.x as i32;
    let py = 2 * (point.y as i32);

    let x = (px + py) / TILE_W;
    let y = -(px - py) / TILE_W;

    //TODO
    // (x, y, 0) is the base location. disambiguate the location.
    // for z in (0..NUM_Z_LEVELS).rev()
    // {
    //     let loc = Tile {
    //         x: x - z,
    //         y: y + z,
    //         z: z
    //     };
    //     if isSelectable(loc) {
    //         return loc;
    //     }
    // }

    Tile { x, y, z: 0 }
}

fn screen_to_xy(screen: Vec2, origin: Vec2, scale: f32) -> Vec2 {
    Vec2 {
        x: (screen.x / scale) + origin.x,
        y: (screen.y / scale) + origin.y,
    }
}

/// Calculate the minimum diamond of iso coordinates that will bound a pair of xy-points.
/// (This is used to determine visible isos.)
fn min_iso_bounding_box_for_xy(p: (Vec2, Vec2)) -> (Tile, Tile) {
    let origin = (p.0.x.min(p.1.x) as i32, p.0.y.min(p.1.y) as i32 * 2);
    let extent = (p.0.x.max(p.1.x) as i32, p.0.y.max(p.1.y) as i32 * 2);

    // Calculate tile coordinates using the same formulas in xy_to_iso()
    let left = (origin.0 + origin.1) / TILE_W;
    let right = (extent.0 + extent.1) / TILE_W + 1; // +1 for safety
    let top = -(extent.0 - origin.1) / TILE_W;
    let bottom = -(origin.0 - extent.1) / TILE_W + 1; // +1 for safety

    (
        Tile {
            x: left,
            y: top,
            z: 0,
        },
        Tile {
            x: right,
            y: bottom,
            z: 0,
        },
    )
}

/// Size of the x,y bounding box that will cover w x h tiles.
fn min_xy_bounding_box_for_iso_size(w: i32, h: i32) -> (i32, i32) {
    ((w + h) * TILE_W_HALF, (w + h) * TILE_H_HALF)
}
