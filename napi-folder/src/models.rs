use std::path::{PathBuf};
use std::time::SystemTime;
// use serde_json;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, skip_serializing_none};
use ts_rs::TS;
#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum MetaType {
    Size,
    Tm,
    HasItems,
}

#[allow(dead_code)]
#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum OrderBy {
    Dir,
    Name,
    Size,
    Tm,
    Mime,
    Ext,
}

#[allow(dead_code)]
#[derive(TS, Serialize, Deserialize, Eq, Clone, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum OrderAsc {
    Asc,
    Desc,
}


#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
#[ts(export)]
pub struct OrdItem {
    pub nm: OrderBy,
    pub asc: OrderAsc,
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CacheKey {
    pub path: String,
    pub tm: SystemTime,
}


#[derive(Clone)]
pub struct CacheVal {
    pub paths: Vec<PathBuf>,
    pub items: Vec<Item>,
    pub ordering: Vec<OrdItem>,
}

#[allow(dead_code)]
#[skip_serializing_none]
#[derive(TS, Serialize, Debug, Default)]
#[ts(export)]
pub struct Folder {
    pub item: Item,
    pub path_param: String,
    pub base_dir: String,
    #[ts(optional)]
    pub tot: Option<usize>,
    #[ts(optional)]
    pub skip_n: Option<usize>,
    #[ts(optional)]
    pub take_n: Option<usize>,
    #[ts(optional)]
    pub ordering: Option<Vec<OrdItem>>,
}


#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(TS, Serialize, Clone, Debug, Default)]
#[ts(export)]
pub struct Item {
    pub name: String,
    pub is_dir: bool,
    #[ts(optional)]
    pub ext: Option<String>,
    #[ts(optional)]
    pub mime: Option<String>,
    #[ts(optional)]
    pub size: Option<u64>,  // u64
    #[ts(optional)]
    pub cnt: Option<usize>,  // usize
    #[ts(optional)]
    pub has_items: Option<bool>,
    #[ts(optional)]
    pub tm: Option<u64>,  // u64
    #[ts(optional)]
    pub items: Option<Vec<Item>>
}

#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(TS, Serialize, Deserialize, Clone, Debug, Default)]
#[ts(export)]
pub struct OptParams {
    pub path_str: Option<String>,
    pub meta_types: Option<Vec<MetaType>>,
    pub ordering: Option<Vec<OrdItem>>,
    pub skip_n: Option<usize>,
    pub take_n: Option<usize>,
    pub is_pretty: Option<bool>,
    pub is_cache: Option<bool>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Params {
    pub path_str: String,
    pub meta_types: Vec<MetaType>,
    pub ordering: Vec<OrdItem>,
    pub skip_n: Option<usize>,
    pub take_n: Option<usize>,
    pub is_pretty: bool,
    pub is_cache: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            path_str: String::from("."),
            meta_types: vec![MetaType::HasItems, MetaType::Size, MetaType::Tm],
            ordering: vec![OrdItem{nm: OrderBy::Dir, asc: OrderAsc::Asc}, OrdItem{nm: OrderBy::Name, asc: OrderAsc::Asc}],
            skip_n: None,
            take_n: Some(5),
            is_pretty: true,
            is_cache: true,
        }
    }
}