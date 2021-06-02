use std::path::PathBuf;
use worldlib::map::WorldMap;
use crate::world::{SerializedMapList, MapConfig};

pub fn load_map_list(root_path: &PathBuf, list: SerializedMapList) -> Vec<WorldMap> {

    println!("    Loading map set \"{}\"", list.identifier);

    let mut maps = Vec::with_capacity(list.dirs.len());

    for dir_string in list.dirs {
        let map_path = root_path.join(dir_string);
        for dir_entry in std::fs::read_dir(&map_path).unwrap_or_else(|err| panic!("Could not read map set directory at {:?} with error {}", map_path, err)) {
            let file = dir_entry.unwrap_or_else(|err| panic!("Could not read map set directory entry at {:?} with error {}", map_path, err)).path();
            if let Some(ext) = file.extension() {
                if ext == std::ffi::OsString::from("ron") {
                    let config: MapConfig = ron::from_str(
                        &std::fs::read_to_string(&file).unwrap_or_else(|err| panic!("Could not read map set configuration at {:?} to string with error {}", file, err))
                    ).unwrap_or_else(|err| panic!("Could not deserialize map set configuration at {:?} with error {}", file, err));
                    println!("        Loaded map set map \"{}\"", config.name);
                    let map = super::load_map_from_config(&map_path, config, Some(list.identifier));
                    maps.push(map);
                }
            }
        }        
    }

    maps

}