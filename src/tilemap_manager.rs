pub mod tilemap_manager {
    use crate::texture_manager::texture_manager::DrawableTileData;
    use obj_pool::{ObjId, ObjPool};
    use std::vec;

    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct Tile {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct MapData {
        pub ground_id: usize,
        pub entity_id: usize,
    }

    #[derive(Debug)]
    pub struct Entity {
        pub x0: usize,
        pub y0: usize,
        pub entity_type_id: usize,
    }

    pub struct TileMap<'a> {
        data: Vec<Vec<MapData>>,
        ground_drawables: Vec<DrawableTileData<'a>>,
        entity_drawables: Vec<DrawableTileData<'a>>,
        entities: ObjPool<Entity>,
    }

    impl<'a> TileMap<'a> {
        pub fn new(size_x: usize, size_y: usize) -> TileMap<'a> {
            let empty = MapData {
                ground_id: 0,
                entity_id: 0,
            };

            let mut t = TileMap {
                data: vec![vec!(empty; size_x); size_y],
                ground_drawables: vec![],
                entity_drawables: vec![],
                entities: ObjPool::new(),
            };

            t.entities.insert(Entity {
                x0: 0,
                y0: 0,
                entity_type_id: 0,
            });

            t
        }

        #[inline]
        fn get(&self, x: usize, y: usize) -> MapData {
            self.data[x][y]
        }

        pub fn get_ground(&self, x: usize, y: usize) -> &DrawableTileData<'a> {
            let idx = self.get(x, y).ground_id;
            &self.ground_drawables[idx]
        }

        pub fn get_entity(&self, x: usize, y: usize) -> Option<&DrawableTileData<'a>> {
            let id = self.get(x, y).entity_id;
            if id > 0 {
                let id = ObjId::from_index(id as u32);
                let entity = self.entities.get(id).unwrap();

                return match entity.x0 == x && entity.y0 == y {
                    true => Some(&self.entity_drawables[entity.entity_type_id]),
                    false => None,
                };
            }
            None
        }

        pub fn create_ground_type(&mut self, drawable: DrawableTileData<'a>) -> usize {
            self.ground_drawables.push(drawable);
            self.ground_drawables.len() - 1
        }

        pub fn set_ground(&mut self, x: usize, y: usize, ground_id: usize) -> bool {
            match self.ground_drawables.get(ground_id) {
                Some(_) => {
                    self.data[x][y].ground_id = ground_id;
                    true
                }
                None => false,
            }
        }

        pub fn create_entity_types(&mut self, drawables: &mut Vec<DrawableTileData<'a>>) {
            self.entity_drawables.append(drawables);
        }

        fn create_entity(&mut self, entity: Entity) -> usize {
            let id = self.entities.insert(entity);
            ObjPool::<usize>::obj_id_to_index(id) as usize
        }

        pub fn set_entity(&mut self, x0: usize, y0: usize, entity_type: usize) -> bool {
            let drawable = &self.entity_drawables[entity_type];

            let x1 = match x0.checked_sub((drawable.size.x - 1) as usize) {
                Some(x1) => x1,
                None => return false,
            };

            let y1 = match y0.checked_sub((drawable.size.y - 1) as usize) {
                Some(y1) => y1,
                None => return false,
            };

            // Check that there is no existing entity within the area
            for x in x1..=x0 {
                for y in y1..=y0 {
                    match self.data[x][y].entity_id {
                        0 => continue,
                        _ => return false,
                    }
                }
            }

            let entity = Entity {
                x0,
                y0,
                entity_type_id: entity_type,
            };
            let id = self.create_entity(entity);

            // Set the area tiles
            for x in x1..=x0 {
                for y in y1..=y0 {
                    self.data[x][y].entity_id = id;
                }
            }

            true
        }
    }
}
