pub mod plugin_manager {
    use crate::tilemap_manager::tilemap_manager::Tile;
    use encoding_rs::*;
    use macroquad::prelude::{Color, BLACK};
    use roxmltree::{Error, Node, ParsingOptions};
    use std::{collections::HashMap, fs, io, path::PathBuf};

    #[derive(Debug)]
    pub struct Plugin {
        pub filename: PathBuf,
        pub title: String,
        pub author: String,
        pub contributions: Vec<Contribution>,
    }

    #[derive(Debug)]
    pub struct Contribution {
        pub size: Tile,
        pub image_ref: String,
        pub image_data: Vec<ContributionImageData>,
        pub color_mappings: Vec<ColorMapping>,
    }

    #[derive(Debug)]
    pub enum ContributionImageData {
        ContributionSprite(ContributionSprite),
        ContributionMultistorey(ContributionMultistorey),
        ContributionAutotile(ContributionAutotile, usize)
    }

    #[derive(Debug)]
    pub struct ContributionMultistorey {
        pub top: ContributionSprite,
        pub middle: ContributionSprite,
        pub bottom: ContributionSprite,
    }

    #[derive(Debug)]
    pub struct ContributionAutotile {

    }

    #[derive(Debug)]
    pub struct ContributionSprite {
        pub origin_x: i32,
        pub origin_y: i32,
        pub offset: i32,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ColorMapping {
        pub target: Color,
        pub channel: ColorMappingChannel,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ColorMappingChannel {
        None,
        Red,
        Green,
        Blue,
    }

    #[derive(Debug)]
    pub enum PluginError {
        ParseError(Error),
    }

    pub fn enumerate_plugins() -> Result<Vec<PathBuf>, io::Error> {
        Ok(fs::read_dir("./plugin")?
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap().path())
            .filter(|r| r.is_dir())
            .collect::<Vec<PathBuf>>())
    }

    pub fn load_plugins(plugin_paths: Vec<PathBuf>) -> Vec<Plugin> {
        let mut plugins = Vec::new();

        for path in plugin_paths {
            let mut xml = path.clone();
            xml.push("plugin.xml");

            let xml_bytes = match fs::read(xml) {
                Ok(file) => file,
                Err(_) => {
                    println!(
                        "Warning: Plugin {} does not have a root file",
                        path.display()
                    );
                    continue;
                }
            };

            let (mut xml_data, _, had_errors) = UTF_8.decode(&xml_bytes);
            if had_errors {
                (xml_data, _, _) = SHIFT_JIS.decode(&xml_bytes);
            }

            let res = parse_plugin_xml(path, &xml_data);

            match res {
                Ok(plugin) => plugins.push(plugin),
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            }
        }

        plugins
    }

    pub fn parse_plugin_xml(filename: PathBuf, data: &str) -> Result<Plugin, PluginError> {
        let options = ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };

        let doc = match roxmltree::Document::parse_with_options(&data, options) {
            Ok(doc) => doc,
            Err(err) => return Err(PluginError::ParseError(err)),
        };

        let mut metadata = HashMap::new();
        let mut contributions = Vec::new();

        let root = doc.descendants().find(|x| x.tag_name().name() == "plug-in");
        match root {
            Some(root) => {
                let metadata_nodes = root
                    .children()
                    .filter(|x| x.is_element() && x.tag_name().name() != "contribution");

                for node in metadata_nodes {
                    let (k, v) = parse_metadata_field(node);
                    metadata.insert(k, v);
                }

                println!(
                    "Found plugin '{}' by {}",
                    metadata["title"], metadata["author"]
                );

                let pictures = root
                    .children()
                    .filter(|x| x.tag_name().name() == "contribution")
                    .filter(|y| y.attribute("type").unwrap_or("") == "picture");

                let mut picture_contributions = HashMap::new();
                for picture in pictures {
                    let (k, v) = parse_picture_contribution(picture);
                    picture_contributions.insert(k, v);
                }

                let other_contributions = root
                    .children()
                    .filter(|x| x.tag_name().name() == "contribution")
                    .filter(|y| y.attribute("type").unwrap_or("picture") != "picture");

                for other in other_contributions {
                    contributions.push(parse_contribution(other));
                }

                resolve_contribution_refs(&picture_contributions, &mut contributions);
            }
            None => {
                println!("Plugin not found for {}", filename.display());
            }
        }

        let title = metadata["title"].to_string();
        let author = metadata["author"].to_string();

        Ok(Plugin {
            filename,
            title,
            author,
            contributions,
        })
    }

    fn parse_metadata_field(node: Node) -> (String, String) {
        let key = node.tag_name().name();
        let value = node.first_child().unwrap().text().unwrap_or("").trim();
        return (key.to_string(), value.to_string());
    }

    fn parse_picture_contribution(node: Node) -> (String, String) {
        let id = node.attribute("id").unwrap().to_string();

        let picture_node = node.children().find(|x| x.tag_name().name() == "picture");
        if picture_node.is_none() {
            //error
            todo!()
        }

        let picture_node = picture_node.unwrap();
        if !picture_node.has_attribute("src") {
            //error
            todo!()
        }

        let src = picture_node.attribute("src").unwrap().to_string();
        (id, src)
    }

    fn parse_contribution(node: Node) -> Contribution {
        match node.attribute("type") {
            Some(contrib_type) => match contrib_type {
                "GenericStructure" => {
                    return parse_generic_structure_contribution(node);
                },
                "road" => {
                    return parse_road_contribution(node);
                }
                other => panic!("Invalid or unimplemented contribution type '{}'", other),
            },
            None => todo!(),
        }
    }

    fn parse_metadata(node: Node) -> HashMap<String, String> {
        let metadata_nodes = node.children().filter(|x| {
            x.is_element() && x.has_children() && x.children().all(|y| !y.is_element())
        });

        let mut metadata = HashMap::new();
        for metadata_node in metadata_nodes {
            let (k, v) = parse_metadata_field(metadata_node);
            metadata.insert(k, v);
        }
        metadata
    }

    fn parse_generic_structure_contribution(node: Node) -> Contribution {
        let metadata = parse_metadata(node);

        let mut color_mappings = parse_hue_transform_nodes(node);
        if color_mappings.len() == 0 {
            color_mappings.push(ColorMapping {
                channel: ColorMappingChannel::None,
                target: BLACK,
            });
        };

        let (sprites, sprite_ref) = parse_generic_structure_sprite(node);
        let (pictures, picture_ref) = parse_generic_structure_multi(node);

        let image_data = match (sprites.len(), pictures.len()) {
            (_, 0) => sprites
                .into_iter()
                .map(|x| ContributionImageData::ContributionSprite(x))
                .collect::<Vec<ContributionImageData>>(),

            (0, _) => pictures
                .into_iter()
                .map(|x| ContributionImageData::ContributionMultistorey(x))
                .collect::<Vec<ContributionImageData>>(),

            _ => panic!("No image data found"),
        };

        let image_ref = match (sprite_ref.is_empty(), picture_ref.is_empty()) {
            (false, true) => sprite_ref,
            (true, false) => picture_ref,
            (false, false) => panic!("Too many image reference data types"),
            (true, true) => panic!("No image reference data found"),
        };

        let size = &metadata["size"];
        let sizes: Vec<_> = size.split(",").collect();
        // x and y are flipped in our coordinate system
        let size_x: i32 = sizes[1].parse().unwrap_or(0);
        let size_y: i32 = sizes[0].parse().unwrap_or(0);

        let height = match metadata.get("height") {
            Some(h) => h.parse().unwrap_or(0),
            None => 0,
        };

        Contribution {
            size: Tile {
                x: size_x,
                y: size_y,
                z: height,
            },
            image_data,
            image_ref,
            color_mappings,
        }
    }

    fn parse_road_contribution(node: Node) -> Contribution {
        // let metadata = parse_metadata(node);

        let sprite_node = node
        .children()
        .find(|x| x.is_element() && x.tag_name().name() == "picture");

        let image_ref: String;

        match sprite_node {
            Some(sprite) => {
                let src = sprite.attribute("src").expect("No image data found!");
                let size = sprite.attribute("size").unwrap_or("32,16");
                let offset = sprite.attribute("offset").unwrap_or("0");

                image_ref = src.to_owned();
                println!("{} {} {}", src, size, offset);
            },
            None => panic!("No image data found!")
        }

        Contribution {
            size: Tile {
                x: 1,
                y: 1,
                z: 1,
            },
            image_ref,
            image_data: vec![],
            color_mappings: vec![]
        }
    }

    fn parse_hue_transform_nodes(node: Node) -> Vec<ColorMapping> {
        let hue_transform_nodes = node.children().filter(|x| {
            x.is_element()
                && x.tag_name().name() == "spriteType"
                && x.has_attribute("name")
                && x.attribute("name").unwrap() == "hueTransform"
        });

        let mut color_mappings = Vec::new();

        for hue_transform_node in hue_transform_nodes {
            let mapping = match hue_transform_node
                .children()
                .find(|x| x.tag_name().name() == "map")
            {
                Some(map) => {
                    if map.has_attribute("from") && map.has_attribute("to") {
                        let from = map.attribute("from").unwrap();
                        let to = map.attribute("to").unwrap();

                        let from_elements: Vec<_> = from.split(",").collect();
                        let to_elements: Vec<_> = to.split(",").collect();

                        match from_elements.len() {
                            3 => match to_elements.len() {
                                3 => {
                                    let channel = match from_elements.iter().position(|x| *x == "*")
                                    {
                                        Some(idx) => match idx {
                                            0 => ColorMappingChannel::Red,
                                            1 => ColorMappingChannel::Green,
                                            2 => ColorMappingChannel::Blue,
                                            _ => continue,
                                        },
                                        None => continue,
                                    };

                                    let target = match parse_target_color_for_mapping(&to_elements)
                                    {
                                        Ok(t) => t,
                                        Err(_) => continue,
                                    };

                                    ColorMapping { channel, target }
                                }
                                _ => continue,
                            },
                            1 => {
                                let channel = match from {
                                    "red" | "Red" | "r" | "R" => ColorMappingChannel::Red,
                                    "blue" | "Blue" | "b" | "B" => ColorMappingChannel::Blue,
                                    "green" | "Green" | "g" | "G" => ColorMappingChannel::Green,
                                    _ => continue,
                                };

                                let target = match parse_target_color_for_mapping(&to_elements) {
                                    Ok(t) => t,
                                    Err(_) => continue,
                                };

                                match to_elements.len() {
                                    3 => ColorMapping { channel, target },
                                    _ => continue,
                                }
                            }
                            _ => continue,
                        }
                    } else {
                        println!(
                            "A hue transform mapping doesn't have 'from' and 'to' properties."
                        );
                        continue;
                    }
                }
                None => {
                    println!("A hue transform node doesn't have a <map> element.");
                    continue;
                }
            };

            color_mappings.push(mapping);
        }

        color_mappings
    }

    fn parse_target_color_for_mapping(
        elements: &Vec<&str>,
    ) -> Result<Color, std::num::ParseFloatError> {
        let r = elements[0].parse::<f32>()? / 255.0;
        let g = elements[1].parse::<f32>()? / 255.0;
        let b = elements[2].parse::<f32>()? / 255.0;

        Ok(Color { r, g, b, a: 1.0 })
    }

    fn parse_origin_and_offset(node: Node) -> (i32, i32, i32) {
        let origin = node.attribute("origin").unwrap(); //todo
        let origins: Vec<_> = origin.split(",").collect();
        let origin_x: i32 = origins[0].trim().parse().unwrap_or(0);
        let origin_y: i32 = origins[1].trim().parse().unwrap_or(0);

        let offset = node.attribute("offset").unwrap(); //todo
        let offset = offset.parse().unwrap();

        (origin_x, origin_y, offset)
    }

    fn parse_generic_structure_sprite(node: Node) -> (Vec<ContributionSprite>, String) {
        let sprite_nodes = node
            .children()
            .filter(|x| x.is_element() && x.tag_name().name() == "sprite");

        let mut image_ref: String = String::new();

        let mut sprites = Vec::new();
        for sprite_node in sprite_nodes {
            let (origin_x, origin_y, offset) = parse_origin_and_offset(sprite_node);

            let picture_node = sprite_node
                .children()
                .find(|x| x.is_element() && x.tag_name().name() == "picture")
                .unwrap();

            match picture_node.attribute("ref") {
                Some(x) => {
                    if image_ref.is_empty() {
                        image_ref = x.to_string()
                    } else {
                        if x != image_ref {
                            panic!("Sprites on a contribution have different image refs.");
                        }
                    }
                }
                None => (),
            }

            sprites.push(ContributionSprite {
                origin_x,
                origin_y,
                offset,
            });
        }
        (sprites, image_ref)
    }

    fn parse_generic_structure_multi(node: Node) -> (Vec<ContributionMultistorey>, String) {
        let picture_nodes = node
            .children()
            .filter(|x| x.is_element() && x.tag_name().name() == "pictures");
        
        let mut image_ref: String = String::new();

        let mut sprites = Vec::new();
        for picture_node in picture_nodes {

            let top_node = picture_node.children().find(|x| x.has_tag_name("top")).unwrap();
            let top = parse_origin_and_offset(top_node);

            let middle_node = picture_node.children().find(|x| x.has_tag_name("middle")).unwrap();
            let middle = parse_origin_and_offset(middle_node);
            
            let bottom_node = picture_node.children().find(|x| x.has_tag_name("bottom")).unwrap();
            let bottom = parse_origin_and_offset(bottom_node);

            let top_picture = top_node.children().find(|x| x.has_tag_name("picture")).unwrap();

            match top_picture.attribute("ref") {
                Some(x) => {
                    if image_ref.is_empty() {
                        image_ref = x.to_string()
                    } else {
                        if x != image_ref {
                            //TODO: check the middle and bottom refs
                            panic!("Sprites on a contribution have different image refs.");
                        }
                    }
                }
                None => (),
            }

            sprites.push(ContributionMultistorey {
                top: ContributionSprite {
                    origin_x: top.0,
                    origin_y: top.1,
                    offset: top.2 
                },
                middle: ContributionSprite {
                    origin_x: middle.0,
                    origin_y: middle.1,
                    offset: middle.2 
                },
                bottom: ContributionSprite {
                    origin_x: bottom.0,
                    origin_y: bottom.1,
                    offset: bottom.2 
                }
            });
        }

        (sprites, image_ref)
    }

    fn resolve_contribution_refs(
        picture_contributions: &HashMap<String, String>,
        contributions: &mut Vec<Contribution>,
    ) {
        for contribution in contributions {
            if picture_contributions.contains_key(&contribution.image_ref) {
                contribution.image_ref = picture_contributions[&contribution.image_ref].clone();
            }
        }
    }
}
