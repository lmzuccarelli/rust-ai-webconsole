use surrealkv::{IsolationLevel, Options};

use crate::MAP_LOOKUP;

pub fn get_error(msg: String) -> Box<dyn std::error::Error> {
    Box::from(format!("{}", msg.to_lowercase()))
}

pub fn get_map_item(item: String) -> Result<String, Box<dyn std::error::Error>> {
    let hm = MAP_LOOKUP.lock()?;
    let deploy_res = hm.as_ref().unwrap().get(&item);
    match deploy_res {
        Some(value) => Ok(value.to_owned()),
        None => Err(get_error(format!("item {} not set", item).to_owned())),
    }
}

pub fn get_opts(name: String) -> Result<Options, Box<dyn std::error::Error>> {
    let db_path = get_map_item("db_path".to_owned())?;
    let mut opts = Options::new();
    opts.disk_persistence = true;
    opts.max_value_threshold = 4096;
    opts.max_segment_size = 268_435_456;
    opts.max_compaction_segment_size = 1_073_741_824;
    opts.isolation_level = IsolationLevel::SerializableSnapshotIsolation;
    opts.enable_versions = false;
    opts.max_value_cache_size = 10000;
    opts.dir = format!("{}/{}.kv", db_path, name).into();
    Ok(opts)
}
