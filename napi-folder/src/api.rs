use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::path::Component::Prefix;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::fs;
use std::io::{BufReader, Read};
use chardetng::EncodingDetector;
use mime_guess::from_path;
use encoding_rs::Encoding;
use moka::future::Cache;
use rayon::prelude::*;
use crate::models::{MetaType, OrdItem, OrderAsc, OrderBy, CacheKey, CacheVal, CachePathsKey, CacheFileKey, Item, Folder, Params, ApiError};
use crate::path_ext::PathExt;
use crate::system_time_ext::SystemTimeExt;

static INSTANCE: OnceLock<Api> = OnceLock::new();

pub fn get_instance() -> &'static Api {
    INSTANCE.get_or_init(|| Api::new())
}

pub struct Api {
    cache_folder: Cache<CacheKey, CacheVal>,
    cache_paths: Cache<CachePathsKey, Vec<PathBuf>>,
    cache_txt: Cache<CacheFileKey, String>,
    state: Cache<String, String>,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            cache_folder: Cache::new(100),
            cache_paths: Cache::new(100),
            cache_txt: Cache::new(100),
            state: Cache::new(100),
        }
    }
}

impl Api {

    #[allow(dead_code)]
    pub fn new() -> Self {
        Api {
            cache_folder: Cache::new(100),
            cache_paths: Cache::new(100),
            cache_txt: Cache::new(100),
            state: Cache::new(100),
        }
    }


    async fn get_entries<P: AsRef<Path>>(&self, p: P) -> Result<Vec<PathBuf>, ApiError> {
        let dir = fs::read_dir(p.as_ref())?;
        let entries: Vec<PathBuf> = dir
            .filter_map(|r| {
                match r {
                    Ok(entry) => {
                        Some(entry)
                    },
                    Err(err) => {
                        println!("err: {:?}", err);
                        None
                    }
                }
            })
            .map(|entry| {
                entry.path()
            })
            .collect()
            ;
        Ok(entries)
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
        if !abs.exists() {
            return Err(ApiError::Folder(String::from("Not Exists Path")))
        }
        if abs.is_file() {  // file -> dir
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
        item.dir = abs.is_dir();
        let mut system_time : Option<SystemTime> = None;
        match abs.metadata() {
            Ok(meta) => {
                system_time = meta.modified().ok();
                item.tm = system_time.map(|t|t.to_sec());
                // item.size = Some(meta.len());
            },
            Err(_) => {
                item.tm = None;
                // item.size = None;
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

            let opt_cache_val = self.cache_folder.get(&cache_key).await;

            sorted_items = match opt_cache_val {

                Some(mut cache_val) => {
                    println!("hit cache");

                    if cache_val.ordering != ordering  {
                        let mut items_cache = cache_val.paths.par_iter().map(|entry|as_item(entry, &meta_types)).collect::<Vec<Item>>();
                        sort_items(&mut items_cache, &ordering);
                        cache_val.items = items_cache;
                        self.cache_folder.insert(cache_key.clone(), cache_val).await;
                    }
                    self.cache_folder.get(&cache_key).await.map(|v| v.items).unwrap_or(vec![])
                }
                None => {
                    let cache_paths_key = CachePathsKey {
                        nm: cache_key.clone().nm,
                        path: cache_key.clone().path,
                        tm: cache_key.clone().tm,
                    };
                    let paths = match self.cache_paths.get(&cache_paths_key).await {
                        Some(paths) => paths,
                        None => {
                            let paths = match self.get_entries(&abs).await {
                                Ok(paths) => {
                                    paths
                                },
                                Err(err) => {
                                    println!("err: {:?}", err);
                                    vec![]
                                },
                            };
                            self.cache_paths.insert(cache_paths_key.clone(), paths.clone()).await;
                            paths.clone()
                        },
                    };

                    let mut items_new = paths.par_iter().map(|entry|as_item(entry, &meta_types)).collect::<Vec<Item>>();
                    sort_items(&mut items_new, &ordering);
                    let cache_val = CacheVal {
                        ordering: ordering.clone(),
                        paths,
                        items: items_new,
                    };
                    self.cache_folder.insert(cache_key.clone(), cache_val).await;
                    self.cache_folder.get(&cache_key).await.map(|v| v.items).unwrap_or(vec![])
                }
            };
        } else {
            match self.get_entries(&abs).await {
                Ok(paths) => {
                    sorted_items = paths.par_iter().map(|entry|as_item(entry, &meta_types)).collect::<Vec<Item>>();
                }
                Err(_err) => {
                    sorted_items = vec![]
                }
            }
            sort_items(&mut sorted_items, &ordering);
        }

        let len_items = sorted_items.len();
        let items_sliced: Vec<Item> = sorted_items.iter().skip(skip_n.unwrap_or(0)).take(take_n.unwrap_or(len_items)).cloned().collect();

        folder.skip_n = Some(skip_n.unwrap_or(0));
        folder.take_n = Some(take_n.unwrap_or(len_items));
        folder.ordering = Some(ordering.clone());
        folder.tot = Some(len_items);
        folder.item.cnt = Some(items_sliced.len());
        folder.item.items = Some(items_sliced);

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

    pub async fn read_txt(&self, path_str: &str) -> Result<String, ApiError> {
        const CHUNK_SIZE: usize = 4096;

        // cache
        let p = PathBuf::from(path_str);
        let tm = p.metadata()?.modified()?;
        let cache_file_key = CacheFileKey {
            nm: "file".to_string(),
            path: p.to_string_lossy().into(),
            tm,
        };
        let opt_cache_val = self.cache_txt.get(&cache_file_key).await;
        if let Some(content) = opt_cache_val {
            return Ok(content)
        };

        let file = fs::File::open(p)?;
        let mut reader = BufReader::new(file);

        let mut detector = EncodingDetector::new();
        let mut detect_buf = [0u8; 8192];
        let n = reader.read(&mut detect_buf)?;
        detector.feed(&detect_buf[..n], n == 0);
        let encoding: &Encoding = detector.guess(None, true);

        let mut output = String::new();
        // let (decoded, had_errors) = encoding.decode_without_bom_handling(&detect_buf[..n]);
        let (decoded, _, had_errors) = encoding.decode(&detect_buf[..n]);
        if had_errors {
            return Err(ApiError::Folder("encoding error".to_string()));
        }
        output.push_str(&decoded);

        let mut buffer = [0u8; CHUNK_SIZE];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            let (decoded, had_errors) = encoding.decode_without_bom_handling(&buffer[..n]);
            if had_errors {
                return Err(ApiError::Folder("encoding error".to_string()));
            }
            output.push_str(&decoded);
        }

        self.cache_txt.insert(cache_file_key, output.clone()).await;

        Ok(output)
    }
}

fn as_item(entry: &PathBuf, meta_types: &Vec<MetaType>) -> Item {
    let nm = match entry.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => panic!("Error Filename")
    };
    let dir = entry.is_dir();
    let mut mt: Option<String> = None;
    let mut ext: Option<String> = None;
    let mut sz: Option<u64> = None;
    let mut tm: Option<u64> = None;
    let mut has: Option<bool> = None;

    let cnt: Option<usize> = None;
    if !dir {
        if meta_types.contains(&MetaType::Mt) {
            mt = Some(from_path(&nm).first_or_octet_stream().to_string());
        }
        if meta_types.contains(&MetaType::Ext) {
            ext = entry.extension().map(|ext| ext.to_string_lossy().to_string().to_lowercase());
        }
    }

    if !meta_types.is_empty() {
        match entry.metadata() {
            Ok(meta) => {
                if dir {
                    if meta_types.contains(&MetaType::Has) {
                        has = Some(entry.has_children());
                    }
                } else {
                    if meta_types.contains(&MetaType::Sz) {
                        sz = Some(meta.len());
                    }
                }
                if meta_types.contains(&MetaType::Tm) {
                    tm = meta.modified().map(|t|t.to_sec()).ok();
                }

            },
            Err(err) => {
                // size = None;
                println!("err: {:?}", err);
            },
        };
    }

    Item {
        nm,
        dir,
        mt,
        ext,
        sz,
        cnt,
        has,
        tm,
        items: None,
    }
}

fn cmp_item<T: Ord>(a: &T, b: &T, asc: &OrderAsc) -> Option<Ordering> {
    if a.ne(&b) {
        return if asc == &OrderAsc::Asc {
            Some(a.cmp(b))
        } else {
            Some(b.cmp(a))
        }
    }
    None
}

fn sort_items(items: &mut Vec<Item>, ordering: &Vec<OrdItem>) {
    items.sort_by(|a, b| {
        for ord in ordering.iter() {
            if OrderBy::Dir == ord.nm {
                match cmp_item(&b.dir, &a.dir, &ord.asc) {
                    Some(ord) => return ord,
                    None => continue
                }
            } else if OrderBy::Nm == ord.nm {
                let a_ord = a.nm.to_lowercase();
                let b_ord = b.nm.to_lowercase();
                match cmp_item(&a_ord, &b_ord, &ord.asc) {
                    Some(ord) => return ord,
                    None => continue
                }
            } else if OrderBy::Ext == ord.nm && !a.dir {
                match (&a.ext, &b.ext) {
                    (Some(a), Some(b)) => {
                        let a_ord = a.to_lowercase();
                        let b_ord = b.to_lowercase();
                        match cmp_item(&a_ord, &b_ord, &ord.asc) {
                            Some(ord) => return ord,
                            None => continue
                        }
                    }
                    _ => continue
                }
            } else if OrderBy::Mt == ord.nm && !a.dir {
                match (&a.mt, &b.mt) {
                    (Some(a), Some(b)) => {
                        let a_ord = a.to_lowercase();
                        let b_ord = b.to_lowercase();
                        match cmp_item(&a_ord, &b_ord, &ord.asc) {
                            Some(ord) => return ord,
                            None => continue
                        }
                    }
                    _ => continue
                }
            } else if OrderBy::Sz == ord.nm && a.sz.ne(&b.sz) {
                match (&a.sz, &b.sz) {
                    (Some(a), Some(b)) => {
                        match cmp_item(a, b, &ord.asc) {
                            Some(ord) => return ord,
                            None => continue
                        }
                    }
                    _ => continue
                }
            }
        }
        if !ordering.iter().any(|o| o.nm == OrderBy::Nm) {
            return a.nm.cmp(&b.nm)
        }
        return Ordering::Equal
    });
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
            path_str: String::from(r"C:\Windows\WinSxS"),
            ..Params::default()
        };
        assert!(api.get_folder(&params).await.is_ok());
        // assert!(api.dir(String::from("C://"), vec![], vec![], None, None).await.is_ok());

    }



    #[tokio::test]
    async fn test_entries() {
        let api = Api::default();
        let s = r"C:\Windows\WinSxS";
        // let s = r"C://MSOCache";
        // let s = r"C://";
        // assert!(api.get_items(PathBuf::from(s)).await.is_err());
        assert!(api.get_entries(PathBuf::from(s)).await.is_ok());
    }


    #[tokio::test]
    async fn test_dir() {
        let api = Api::default();
        let params = Params {
            path_str: String::from(r"C:\Windows\WinSxS"),
            ..Params::default()
        };
        assert!(api.get_folder(&params).await.is_ok());

    }

    #[tokio::test]
    async fn test_get_folder() {
        let api = Api::default();
        let params = Params {
            path_str: String::from(r"C:\Windows\WinSxS"),
            // path_str: String::from(r"kkk"),
            // is_cache: false,
            ..Params::default()
        };
        match api.get_folder(&params).await {
            Ok(_) => {
                println!("ok");
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


    }