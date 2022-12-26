pub mod plugin_manager {
    use encoding_rs::*;
    use roxmltree::{Node, ParsingOptions};
    use std::{collections::HashMap, fs, io, path::PathBuf};

    #[derive(Debug)]
    struct Contribution {
        size_x: i32,
        size_y: i32,
        height: i32,
        sprites: Vec<ContributionSprite>,
    }

    #[derive(Debug)]
    struct ContributionSprite {
        origin_x: i32,
        origin_y: i32,
        offset: i32,
        picture_ref: String,
    }

    pub fn enumerate_plugins() -> Result<Vec<PathBuf>, io::Error> {
        Ok(fs::read_dir("./plugin")?
            .into_iter()
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap().path())
            .filter(|r| r.is_dir())
            .collect::<Vec<PathBuf>>())
    }

    pub fn load_plugins(plugins: Vec<PathBuf>) {
        for plugin in plugins {
            let mut xml = plugin.clone();
            xml.push("plugin.xml");

            let xml_bytes = match fs::read(xml) {
                Ok(file) => file,
                Err(_) => {
                    println!(
                        "Warning: Plugin {} does not have a root file",
                        plugin.display()
                    );
                    continue;
                }
            };

            let (mut xml_data, _, had_errors) = UTF_8.decode(&xml_bytes);
            if had_errors {
                (xml_data, _, _) = SHIFT_JIS.decode(&xml_bytes);
            }

            parse_plugin_xml(&plugin.display().to_string(), &xml_data);
        }
    }

    pub fn parse_plugin_xml(filename: &str, data: &str) {
        let options = ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };

        let doc = match roxmltree::Document::parse_with_options(&data, options) {
            Ok(doc) => doc,
            Err(err) => {
                println!("Error parsing plugin '{}': {}", filename, err);
                return;
            }
        };

        let root = doc.descendants().find(|x| x.tag_name().name() == "plug-in");
        match root {
            Some(root) => {
                let mut metadata = HashMap::new();
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

                let mut contributions = Vec::new();
                for other in other_contributions {
                    contributions.push(parse_contribution(other));
                }

                resolve_contribution_refs(&picture_contributions, &mut contributions);
            }
            None => {
                println!("Plugin not found for {}", filename);
            }
        }
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

        let sprite_nodes = node
            .children()
            .filter(|x| x.is_element() && x.tag_name().name() == "sprite");

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
            let picture_ref = picture_node.attribute("ref").unwrap().to_string(); //todo

            sprites.push(ContributionSprite {
                origin_x,
                origin_y,
                offset,
                picture_ref,
            });
        }

        let size = &metadata["size"];
        let sizes: Vec<_> = size.split(",").collect();
        let size_x: i32 = sizes[0].parse().unwrap_or(0);
        let size_y: i32 = sizes[1].parse().unwrap_or(0);

        let height = metadata["height"].parse().unwrap_or(0);

        Contribution {
            size_x,
            size_y,
            height,
            sprites,
        }
    }

    fn resolve_contribution_refs(
        picture_contributions: &HashMap<String, String>,
        contributions: &mut Vec<Contribution>,
    ) {
        for contribution in contributions {
            for sprite in &mut contribution.sprites {
                sprite.picture_ref = picture_contributions[&sprite.picture_ref].clone();
            }
        }
    }
}
