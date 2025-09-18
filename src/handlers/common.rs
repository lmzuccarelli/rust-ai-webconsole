use custom_logger as log;
use surrealkv::{Tree, TreeBuilder};

use crate::MAP_LOOKUP;

pub fn get_error(msg: String) -> Box<dyn std::error::Error> {
    Box::from(format!("{}", msg.to_lowercase()))
}

pub fn get_map_item(item: String) -> Result<String, Box<dyn std::error::Error>> {
    let hm = MAP_LOOKUP.lock()?;
    let deploy_res = hm.as_ref().unwrap().get(&item);
    match deploy_res {
        Some(value) => Ok(value.to_owned()),
        None => Err(get_error(
            format!("[get_map_item] item {} not set", item).to_owned(),
        )),
    }
}

pub fn get_opts(db: String) -> Result<Tree, Box<dyn std::error::Error>> {
    let db_path = get_map_item("db_path".to_owned())?;
    log::debug!("[get_opts] db_path {}", db_path);
    let tree = TreeBuilder::new()
        .with_path(format!("{}/{}.kv", db_path, db).into())
        .with_max_memtable_size(100 * 1024 * 1024)
        .with_block_size(4096)
        .with_level_count(1);
    let t = tree.build()?;
    log::trace!("[get_opts] tree built");
    Ok(t)
}
