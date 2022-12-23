pub mod plugin_manager {
    use encoding_rs::*;
    use std::{fs, io, path::PathBuf};

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

            let (cow, encoding_used, had_errors) = SHIFT_JIS.decode(&xml_bytes);

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
        }
    }
}
