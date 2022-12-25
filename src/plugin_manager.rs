pub mod plugin_manager {
    use encoding_rs::*;
    use roxmltree::ParsingOptions;
    use std::{borrow::Cow, fs, io, path::PathBuf};

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

            let (mut xml_data, mut encoding_used, mut had_errors) = UTF_8.decode(&xml_bytes);
            if had_errors {
                (xml_data, encoding_used, had_errors) = SHIFT_JIS.decode(&xml_bytes);
            }

            println!(
                "Found plugin file for {} using encoding '{}' ({})",
                plugin.display(),
                encoding_used.name(),
                if had_errors {
                    "with errors"
                } else {
                    "no errors"
                }
            );

            load_plugin_xml(&xml_data);
        }
    }

    pub fn load_plugin_xml(data: &str) {
        let doc = match roxmltree::Document::parse_with_options(
            &data,
            ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        ) {
            Ok(doc) => doc,
            Err(err) => {
                println!("Error parsing plugin. {}", err);
                return;
            }
        };

        let mut plugin_name = "";
        let mut plugin_author = "";

        for node in doc.descendants() {
            if node.is_element() {
                if node.tag_name().name() == "plug-in" {
                    if node.has_children() {
                        for child in node.children() {
                            if child.tag_name().name() == "title" {
                                plugin_name = child.first_child().unwrap().text().unwrap();
                            }
                            if child.tag_name().name() == "author" {
                                plugin_author = child.first_child().unwrap().text().unwrap();
                            }
                        }
                    }
                }
            }
        }

        println!("Found plugin '{}' by {}", plugin_name, plugin_author);
    }
}
