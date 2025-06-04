use std::collections::BTreeSet;
use std::time::SystemTime;
use napi;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, skip_serializing_none};
use ts_rs::TS;
use thiserror::Error;
use std::io;
use serde_json;
use napi::{Error as NApiError, Status};

#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
#[ts(export)]
pub enum MetaType {
    Sz,
    Tm,
    Has,
    Mt,
    Ext,
}

#[allow(dead_code)]
#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum OrderBy {
    Dir,
    Nm,
    Sz,
    Tm,
    Mt,
    Ext,
}

#[allow(dead_code)]
#[derive(TS, Serialize, Deserialize, Eq, Clone, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum OrderAsc {
    Asc,
    Desc,
}

#[allow(dead_code)]
#[derive(TS, Serialize, Deserialize, Eq, Clone, PartialEq, Hash, Debug)]
#[ts(export)]
pub enum HomeType {
    HomeDir,
    DownloadDir,
    VideoDir,
    DocumentDir,
    DesktopDir,
    PictureDir,
    AudioDir,
    ConfigDir,
    DataDir,
    DataLocalDir,
    CacheDir,
    FontDir,
    PublicDir,
    ExecutableDir,
    RuntimeDir,
    TemplateDir,
}


#[derive(TS, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
#[ts(export)]
pub struct OrdItem {
    pub nm: OrderBy,
    pub asc: OrderAsc,
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CacheKey {
    pub nm: String,
    pub path: String,
    pub tm: SystemTime,
    pub meta_types: BTreeSet<MetaType>,
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CachePathsKey {
    pub nm: String,
    pub path: String,
    pub tm: SystemTime,
}


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CacheFileKey {
    pub nm: String,
    pub path: String,
    pub tm: SystemTime,
}

#[derive(Clone)]
pub struct CacheVal {
    pub items: Vec<Item>,
    pub ordering: Vec<OrdItem>,
}

#[allow(dead_code)]
#[skip_serializing_none]
#[derive(TS, Serialize, Debug, Default)]
#[ts(export, optional_fields)]
pub struct Folder {
    pub item: Item,
    pub path_param: String,
    pub base_nm: String,
    pub tot: Option<usize>,
    pub skip_n: Option<usize>,
    pub take_n: Option<usize>,
    pub ordering: Option<Vec<OrdItem>>,
}


#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(TS, Serialize, Clone, Debug, Default)]
#[ts(export, optional_fields)]
pub struct Item {
    pub nm: String,
    pub dir: bool,
    pub ext: Option<String>,
    pub mt: Option<String>,
    pub sz: Option<u64>,  // u64
    pub cnt: Option<usize>,  // usize
    pub has: Option<bool>,
    pub tm: Option<u64>,  // u64
    pub items: Option<Vec<Item>>
}

#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(TS, Serialize, Clone, Debug, Default)]
#[ts(export, optional_fields)]
pub struct TextContent {
    pub path: String,
    pub mimetype: String,
    pub enc: Option<String>,
    pub text: Option<String>,
}


#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(TS, Serialize, Deserialize, Clone, Debug, Default)]
#[ts(export, optional_fields)]
pub struct OptParams {
    pub path_str: Option<String>,
    pub meta_types: Option<Vec<MetaType>>,
    pub ordering: Option<Vec<OrdItem>>,
    pub skip_n: Option<usize>,
    pub take_n: Option<usize>,
    pub is_pretty: Option<bool>,
    pub cache_nm: Option<String>,
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
    pub cache_nm: Option<String>,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            path_str: String::from("."),
            meta_types: vec![MetaType::Has, MetaType::Sz, MetaType::Tm],
            ordering: vec![OrdItem{nm: OrderBy::Dir, asc: OrderAsc::Asc}, OrdItem{nm: OrderBy::Nm, asc: OrderAsc::Asc}],
            skip_n: None,
            take_n: Some(5),
            is_pretty: true,
            cache_nm: None,
        }
    }
}



#[derive(Error, Debug)]
pub enum ApiError {
    #[error("NApi error: {0}")]
    NApi(#[from] napi::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Folder error: {0}")]
    Folder(String),

}

impl From<ApiError> for NApiError {
    fn from(err: ApiError) -> Self {
        match err {
            ApiError::NApi(e) => e,

            ApiError::Io(e) => {
                NApiError::new(Status::GenericFailure, format!("IO error: {}", e))
            }

            ApiError::Json(e) => {
                NApiError::new(Status::InvalidArg, format!("JSON error: {}", e))
            }

            ApiError::Folder(msg) => {
                NApiError::new(Status::GenericFailure, msg)
            }
        }
    }
}

