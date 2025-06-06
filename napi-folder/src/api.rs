use std::{cmp};
use std::collections::HashMap;
use std::path::{PathBuf};
use std::path::Component::Prefix;
use std::sync::OnceLock;
use std::time::SystemTime;
use tokio;
use tokio::io::AsyncReadExt;
use mime_guess::{from_path};
use encoding_rs::Encoding;
use chardetng::EncodingDetector;
use moka::future::Cache;
use dirs_next;
use sysinfo::Disks;

use crate::models::{ CacheKey, CacheVal,
                    Item, Folder, Params, TextContent, ApiError, HomeType, DiskInfo};
use crate::path_ext::PathExt;
use crate::system_time_ext::SystemTimeExt;
use crate::dir::{get_items_win32, update_items, sort_items };

static INSTANCE: OnceLock<Api> = OnceLock::new();

pub fn get_instance() -> &'static Api {
    INSTANCE.get_or_init(|| Api::new())
}

pub struct Api {
    cache_folder: Cache<CacheKey, CacheVal>,
    // cache_paths: Cache<CachePathsKey, Vec<PathBuf>>,
    state: Cache<String, String>,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            cache_folder: Cache::new(100),
            // cache_paths: Cache::new(100),
            state: Cache::new(100),
        }
    }
}

impl Api {

    #[allow(dead_code)]
    pub fn new() -> Self {
        Api {
            cache_folder: Cache::new(100),
            // cache_paths: Cache::new(100),
            state: Cache::new(100),
        }
    }

    pub async fn get_folder(&self, params: &Params) -> Result<Folder, ApiError> {

        let Params {
            path_str,
            meta_types,
            ordering,
            skip_n,
            take_n,
            cache_nm,
            ..
        } = params.clone();

        let mut folder = Folder::default();
        let mut abs = std::path::absolute(PathBuf::from(path_str))?;
        let is_file = abs.is_file();
        if is_file {  // file -> dir
            abs.pop();
        }

        let prefix = if let Some(Prefix(prefix_component)) = abs.components().next() {
            Some(prefix_component.as_os_str().to_string_lossy().to_string())
        } else {
            None
        };
        let base_dir: String;
        let abs_parent = abs.parent().map(PathBuf::from);
        let abs_filename = match abs.file_name() {
            Some(nm) => nm.to_string_lossy().to_string(),
            None => String::from("/")
        };
        let is_parent_root = match abs.parent() {
            Some(parent) => parent.is_root(),
            None => true
        };
        if abs.is_root() || is_parent_root {
            base_dir = prefix.unwrap_or_default();
        } else {
            match abs_parent {
                Some(p) => {
                    abs = p.join(PathBuf::from(abs_filename));
                    base_dir = p.to_string_lossy().to_string();
                }
                None => return Err(ApiError::Folder(String::from("Err Parent"))),
            };
        }

        let item_name = match abs.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => "".to_string(),
        };
        //   param        base_dir     item_name     items
        //   C://          C:            ""
        //   C://abc       C:            "abc"
        //   C://abc/def   C://abc       "def"

        folder.path_param = abs.to_string_lossy().into();
        folder.base_nm = base_dir;

        let mut item = Item::default();
        item.nm = item_name;
        item.dir = !is_file;
        let mut system_time : Option<SystemTime> = None;
        match abs.metadata() {
            Ok(meta) => {
                system_time = meta.modified().ok();
                item.tm = system_time.map(|t|t.to_sec());
            },
            Err(_) => {
                item.tm = None;
            }
        }

        folder.item = item;

        let mut sorted_items: Vec<Item>;

        if let Some(cache_nm_str) = cache_nm {
            let cache_key = CacheKey {
                nm: cache_nm_str,
                path: folder.path_param.clone(),
                tm: match system_time {
                    Some(system_time) => system_time,
                    None => return Err(ApiError::Folder(String::from("Err SystemTime"))),
                },
                meta_types: meta_types.clone().into_iter().collect(),
            };

            sorted_items = match self.cache_folder.get(&cache_key).await {
                Some(mut cache_val) => {
                    if cache_val.ordering != ordering  {
                        // let mut items_cache = get_items_win32(abs.to_string_lossy().as_ref(), &meta_types).unwrap_or(vec![]);
                        // update_items(&mut items_cache, &meta_types);
                        println!("sort");
                        sort_items(&mut cache_val.items, &ordering);
                        // cache_val.items = items_cache;
                        self.cache_folder.insert(cache_key.clone(), cache_val.clone()).await;
                    } else {
                        println!("hit cache folder");
                    }
                    cache_val.items
                }
                None => {
                    println!("read folder");
                    let mut items_new = get_items_win32(abs.to_string_lossy().as_ref(), &meta_types).unwrap_or(vec![]);
                    update_items(&mut items_new, &meta_types);
                    sort_items(&mut items_new, &ordering);

                    let cache_val = CacheVal {
                        ordering: ordering.clone(),
                        items: items_new.clone(),
                    };
                    self.cache_folder.insert(cache_key.clone(), cache_val.clone()).await;
                    items_new
                }
            };
        } else {
            sorted_items = get_items_win32(abs.to_string_lossy().as_ref(), &meta_types).unwrap_or(vec![]);
            sort_items(&mut sorted_items, &ordering);
        }

        let len_items = sorted_items.len();
        let mut skip = skip_n.unwrap_or(0);
        skip = cmp::min(skip, len_items);

        let take = match take_n {
            Some(n) => cmp::min(n, len_items - skip),
            None =>  len_items - skip
        };
        let items_sliced: Vec<Item> = sorted_items.iter().skip(skip).take(take).cloned().collect();
        folder.skip_n = Some(skip);
        folder.take_n = Some(take);
        folder.ordering = Some(ordering.clone());
        folder.tot = Some(len_items);
        folder.cnt = Some(items_sliced.len());
        folder.item.items = Some(items_sliced);
        // folder.item.has = if meta_types.contains(&MetaType::Has) { Some(len_items > 0) } else { None };

        Ok(folder)
    }

    pub async fn set_state(&self, key: String, opt_val: Option<String>) -> Result<Option<String>, ApiError> {
        match opt_val.clone() {
            None => {
                self.state.remove(&key).await;
            },
            Some(val) => {
                self.state.insert(key.clone(), val.clone()).await;
            },
        };
        Ok(opt_val)
    }

    pub async fn get_state(&self, key: &String, default_val: Option<String>) -> Result<Option<String>, ApiError> {
        let opt_val = self.state.get(key).await;
        match (opt_val.clone(), default_val.clone()) {
            (None, Some(val)) => {
                self.state.insert(key.clone(), val.clone()).await;
                Ok(default_val)
            }
            (opt_val, _) => {
                Ok(opt_val)
            }
        }
    }

    pub async fn read_txt(&self, path_str: &str) -> Result<TextContent, ApiError> {
        let path = PathBuf::from(path_str);

        let mut file = tokio::fs::File::open(&path).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut sample = vec![0u8; 16 * 1024];
        let n = reader.read(&mut sample).await?;
        sample.truncate(n);

        let mime_type = match infer::get(&sample) {
            Some(infer_type) => infer_type.mime_type().to_string(),
            None => from_path(path_str).first_or_octet_stream().to_string()
        };

        // let mut mime_type = from_path(path_str).first_or_octet_stream().to_string();
        // if mime_type == "application/octet-stream" {
        //     if let Some(infer_type) = infer::get(&sample) {
        //         mime_type = infer_type.mime_type().to_string()
        //     }
        // }

        println!("mime_type: {}", mime_type);

        // application/octet-stream  인경우 기본적으로 안보이게 하나 file_size가 5M 미만인경우는 열기시도
        let sz = path.metadata()?.len();
        
        if sz > 5 * 1024 * 1024 {
            // return Err(ApiError::Folder(String::from("Err MimeType")))
            Ok(TextContent {
                path: path_str.to_string(),
                mimetype: mime_type,
                enc: None,
                text: None
            })
        } else {
            file = tokio::fs::File::open(&path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;

            let mut detector = EncodingDetector::new();
            detector.feed(&buffer, true);
            let encoding: &Encoding = detector.guess(None, true);

            let (text, _, had_errors) = encoding.decode(&buffer);
            let opt_text = if had_errors {
                None
            } else {
                Some(text.into_owned())
            };

            Ok(TextContent {
                path: path_str.to_string(),
                mimetype: mime_type,
                enc: Some(encoding.name().to_string()),
                text: opt_text
            })
        }
    }

    pub async fn get_home_dir(&self) -> Result<HashMap<HomeType, String>, ApiError> {
        Ok([
            (HomeType::RootDir, Some(std::path::absolute(PathBuf::from("/"))?)),
            (HomeType::HomeDir, dirs_next::home_dir()),
            (HomeType::DownloadDir ,dirs_next::download_dir()),
            (HomeType::VideoDir ,dirs_next::video_dir()),
            (HomeType::DocumentDir ,dirs_next::document_dir()),
            (HomeType::DesktopDir ,dirs_next::desktop_dir()),
            (HomeType::PictureDir ,dirs_next::picture_dir()),
            (HomeType::AudioDir ,dirs_next::audio_dir()),
            (HomeType::ConfigDir ,dirs_next::config_dir()),
            (HomeType::DataDir ,dirs_next::data_dir()),
            (HomeType::DataLocalDir ,dirs_next::data_local_dir()),
            (HomeType::CacheDir ,dirs_next::cache_dir()),
            (HomeType::FontDir ,dirs_next::font_dir()),
            (HomeType::PublicDir ,dirs_next::public_dir()),
            (HomeType::ExecutableDir ,dirs_next::executable_dir()),
            (HomeType::RuntimeDir ,dirs_next::runtime_dir()),
            (HomeType::TemplateDir ,dirs_next::template_dir()),
        ].into_iter().filter_map(|(k, opt) | {
            opt.map(|v| (k, v.to_string_lossy().into_owned()))
        }).collect())
    }
    
    pub async fn get_disks(&self) -> Result<Vec<DiskInfo>, ApiError> {
        let disks = Disks::new_with_refreshed_list();
        let mut ret: Vec<DiskInfo> = vec![];
        for disk in &disks {
            let disk_info = DiskInfo {
                path: disk.mount_point().to_string_lossy().into_owned(),
            };
            ret.push(disk_info);
        }
        Ok(ret)
    }
}




#[cfg(test)]
mod tests {
    // use crate::{models};
    use super::*;


    #[tokio::test]
    async fn test_abc() {

    }

    #[tokio::test]
    async fn test_base() {
        let api = Api::default();
        //   param        base_dir     item_name     items
        //   C://          C:            ""
        //   C://abc       C:            "abc"
        //   C://abc/def   C://abc       "def"

        // let x = api.dir("C://").await;
        // assert_eq!(api.dir(String::from("C://"), vec![], vec![], None, None).await.unwrap().base_dir, "C:");
        // assert_eq!(api.dir(String::from("C://"), vec![], vec![], None, None).await.unwrap().item.name, "");
        let params = Params {
            path_str: String::from(r"C://docs"),
            ..Params::default()
        };
        assert_eq!(api.get_folder(&params).await.unwrap().base_nm, "C:");
    }

    #[tokio::test]
    async fn test_permissions() {
        let api = Api::default();
        let params = Params {
            // path_str: String::from(r"C:\Windows\WinSxS"),
            path_str: String::from(r"C:\"),
            ..Params::default()
        };
        assert!(api.get_folder(&params).await.is_ok());
        // assert!(api.dir(String::from("C://"), vec![], vec![], None, None).await.is_ok());

    }


    #[tokio::test]
    async fn test_dir() {
        let api = Api::default();
        let params = Params {
            // path_str: String::from(r"C:\Windows\WinSxS"),
            path_str: String::from(r"C:\"),
            ..Params::default()
        };
        assert!(api.get_folder(&params).await.is_ok());

    }

    #[tokio::test]
    async fn test_get_folder() {
        let api = Api::default();
        let params = Params {
            // path_str: String::from(r"C:\Windows\WinSxS"),
            path_str: String::from(r"C:\"),
            // path_str: String::from(r"kkk"),
            // is_cache: false,
            ..Params::default()
        };
        match api.get_folder(&params).await {
            Ok(res) => {
                println!("ok:  {:?}", res);
            },
            Err(err) => {
                println!("err: {:?}", err);
            },
        };
        // match api.get_folder(params.clone()).await {
        //     Ok(json) => {
        //         println!("{}", json.len());
        //     },
        //     Err(err) => {
        //         println!("err: {:?}", err);
        //     },
        // };
    }
    #[tokio::test]
    async fn test_state() {
        let api = Api::default();
        let s = api.set_state(String::from("a"), Some(String::from("1"))).await;
        println!("{:?}", s);
        let s = api.get_state(&String::from("a"), None).await;
        println!("{:?}", s);
    }

    #[tokio::test]
    async fn test_get_disks() {
        let api = Api::default();
        let s = api.get_disks().await;
        println!("{:?}", s);
    }


    #[tokio::test]
    async fn test_read_txt() {
        let api = Api::default();
        // let s = r"c:\docs\t1.cp949.txt";
        // let s = r"c:\docs\t1.utf8.txt";
        // let s = r"c:\docs\t1.json";
        let s = r"C:\Users\kkt\Downloads\vite.main.config.ts";
        // let s = r"C:\sources\sample\header-logo.png";
        match api.read_txt(s).await {
            Ok(text_content) => {
                println!("{:?}", text_content);
            },
            Err(err) => {
                println!("err: {:?}", err);
            },
        }
    }

}