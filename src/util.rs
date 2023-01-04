pub mod util {
    use crate::tilemap_manager::tilemap_manager::Tile;
    use macroquad::prelude::Vec2;

    pub const TILE_W: i32 = 32;
    pub const TILE_H: i32 = 16;
    pub const TILE_W_HALF: i32 = TILE_W / 2;
    pub const TILE_H_HALF: i32 = TILE_H / 2;

    pub fn iso_to_xy(tile: &Tile) -> Vec2 {
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

    pub fn xy_to_screen(point: Vec2, origin: Vec2, scale: f32) -> Vec2 {
        Vec2 {
            x: (point.x - origin.x) * scale,
            y: (point.y - origin.y) * scale,
        }
    }

    pub fn xy_to_iso(point: Vec2) -> Tile {
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

    pub fn screen_to_xy(screen: Vec2, origin: Vec2, scale: f32) -> Vec2 {
        Vec2 {
            x: (screen.x / scale) + origin.x,
            y: (screen.y / scale) + origin.y,
        }
    }

    /// Calculate the minimum diamond of iso coordinates that will bound a pair of xy-points.
    /// (This is used to determine visible isos.)
    pub fn min_iso_bounding_box_for_xy(p: (Vec2, Vec2)) -> (Tile, Tile) {
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
    pub fn min_xy_bounding_box_for_iso_size(w: i32, h: i32) -> (i32, i32) {
        ((w + h) * TILE_W_HALF, (w + h) * TILE_H_HALF)
    }
}
