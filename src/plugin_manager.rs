pub mod plugin_manager {
    use encoding_rs::*;
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
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub image_ref: String,
        pub image_data: Vec<ContributionImageData>,
    }

    #[derive(Debug)]
    pub enum ContributionImageData {
        ContributionSprite(ContributionSprite),
        ContributionPictures(ContributionPictures),
    }

    #[derive(Debug)]
    pub struct ContributionSprite {
        pub origin_x: i32,
        pub origin_y: i32,
        pub offset: i32,
    }

    #[derive(Debug)]
    pub struct ContributionPictures {
        pub top: ContributionSprite,
        pub middle: ContributionSprite,
        pub bottom: ContributionSprite,
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
                }
                other => panic!("Invalid or unimplemented contribution type '{}'", other),
            },
            None => todo!(),
        }
    }

    fn parse_generic_structure_contribution(node: Node) -> Contribution {
        let metadata_nodes = node
            .children()
            .filter(|x| x.is_element() && x.children().all(|y| !y.is_element()));

        let mut metadata = HashMap::new();
        for metadata_node in metadata_nodes {
            let (k, v) = parse_metadata_field(metadata_node);
            metadata.insert(k, v);
        }

        let (sprites, sprite_ref) = parse_generic_structure_sprite(node);
        let (pictures, picture_ref) = parse_generic_structure_pictures(node);

        let image_data = match (sprites.len(), pictures.len()) {
            (_, 0) => sprites
                .into_iter()
                .map(|x| ContributionImageData::ContributionSprite(x))
                .collect::<Vec<ContributionImageData>>(),

            (0, _) => pictures
                .into_iter()
                .map(|x| ContributionImageData::ContributionPictures(x))
                .collect::<Vec<ContributionImageData>>(),

            _ => panic!("No image data found"),
        };

        let image_ref = match (sprite_ref.is_empty(), picture_ref.is_empty()) {
            (false, true) => sprite_ref,
            (true, false) => picture_ref,
            _ => panic!("No image reference data found"),
        };

        let size = &metadata["size"];
        let sizes: Vec<_> = size.split(",").collect();
        let size_x: i32 = sizes[0].parse().unwrap_or(0);
        let size_y: i32 = sizes[1].parse().unwrap_or(0);

        let height = metadata["height"].parse().unwrap_or(0);

        Contribution {
            x: size_x,
            y: size_y,
            z: height,
            image_data,
            image_ref,
        }
    }

    fn parse_generic_structure_sprite(node: Node) -> (Vec<ContributionSprite>, String) {
        let sprite_nodes = node
            .children()
            .filter(|x| x.is_element() && x.tag_name().name() == "sprite");

        let mut image_ref: String = String::new();

        let mut sprites = Vec::new();
        for sprite_node in sprite_nodes {
            let offset = sprite_node.attribute("offset").unwrap(); //todo
            let offset = offset.parse().unwrap();

            let origin = sprite_node.attribute("origin").unwrap(); //todo
            let origins: Vec<_> = origin.split(",").collect();
            let origin_x: i32 = origins[0].parse().unwrap_or(0);
            let origin_y: i32 = origins[1].parse().unwrap_or(0);

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

    fn parse_generic_structure_pictures(node: Node) -> (Vec<ContributionPictures>, String) {
        (Vec::new(), String::new())
    }

    fn resolve_contribution_refs(
        picture_contributions: &HashMap<String, String>,
        contributions: &mut Vec<Contribution>,
    ) {
        for contribution in contributions {
            contribution.image_ref = picture_contributions[&contribution.image_ref].clone();
        }
    }
}
