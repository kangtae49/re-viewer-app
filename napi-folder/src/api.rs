use std::cmp::Ordering;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::path::Component::Prefix;
use std::sync::OnceLock;
use std::time::SystemTime;
use mime_guess::from_path;
use moka::future::Cache;
use rayon::prelude::*;
use crate::models::{MetaType, OrdItem, OrderAsc, OrderBy, CacheKey, CacheVal, Item, Folder, Params};
use crate::path_ext::PathExt;
use crate::system_time_ext::SystemTimeExt;

static INSTANCE: OnceLock<Api> = OnceLock::new();

pub fn get_instance() -> &'static Api {
    INSTANCE.get_or_init(|| Api::new())
}

pub struct Api {
    cache: Cache<CacheKey, CacheVal>,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            cache: Cache::new(100),
        }
    }
}

impl Api {

    #[allow(dead_code)]
    pub fn new() -> Self {
        Api {
            cache: Cache::new(100),
        }
    }

    #[allow(dead_code)]
    pub async fn get_folder(&self, params: Params) -> io::Result<String> {
        let Params {
            is_pretty,
            ..
        } = params;
        match self.dir(params).await {
            Ok(folder) => {
                let json = if is_pretty {
                    serde_json::to_string_pretty(&folder)?
                } else {
                    serde_json::to_string(&folder)?
                };
                Ok(json)
            },
            Err(err) => {
                println!("err: {:?}", err);
                Err(err)
            },
        }
    }

    async fn get_entries<P: AsRef<Path>>(&self, p: P) -> io::Result<Vec<PathBuf>> {
        let dir = std::fs::read_dir(p.as_ref())?;
        // let mut dir = fs::read_dir(p.as_ref()).await?;
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


    async fn dir(&self, params: Params) -> io::Result<Folder> {
        let Params {
            path_str,
            meta_types,
            ordering,
            skip_n,
            take_n,
            is_cache,
            ..
        } = params;

        let mut folder = Folder::default();
        let mut abs = std::path::absolute(PathBuf::from(path_str))?;
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
                None => return Err(io::Error::new(ErrorKind::Other, String::from("Err Path"))),
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
        folder.base_dir = base_dir;

        let mut item = Item::default();
        item.name = item_name;
        item.is_dir = abs.is_dir();
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
        if is_cache {
            let cache_key = CacheKey {
                path: folder.path_param.clone(),
                tm: match system_time {
                    Some(system_time) => system_time,
                    None => return  Err(io::Error::new(ErrorKind::Other, "Err SystemTime")),
                },
            };

            let opt_cache_val = self.cache.get(&cache_key).await;
            sorted_items = match opt_cache_val {
                Some(mut cache_val) => {
                    println!("hit cache");
                    if &cache_val.ordering != &ordering {
                        let mut items_cache = cache_val.paths.par_iter().map(|entry|as_item(entry, &meta_types)).collect::<Vec<Item>>();
                        sort_items(&mut items_cache, ordering.clone());
                        cache_val.items = items_cache;
                        self.cache.insert(cache_key.clone(), cache_val).await;
                    }
                    self.cache.get(&cache_key).await.map(|v| v.items).unwrap_or(vec![])
                }
                None => {
                    match self.get_entries(&abs).await {
                        Ok(paths) => {
                            let mut items_new = paths.par_iter().map(|entry|as_item(entry, &meta_types)).collect::<Vec<Item>>();
                            sort_items(&mut items_new, ordering.clone());
                            let cache_val = CacheVal {
                                ordering: ordering.clone(),
                                paths,
                                items: items_new,
                            };
                            self.cache.insert(cache_key.clone(), cache_val).await;
                            self.cache.get(&cache_key).await.map(|v| v.items).unwrap_or(vec![])
                        }
                        Err(err) => {
                            println!("err: {:?}", err);
                            vec![]
                        },
                    }
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
            sort_items(&mut sorted_items, ordering.clone());
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
}

fn as_item(entry: &PathBuf, meta_types: &Vec<MetaType>) -> Item {
    let name = match entry.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => panic!("Error Filename")
    };
    let is_dir = entry.is_dir();
    let mut mime: Option<String> = None;
    let mut ext: Option<String> = None;
    let mut size: Option<u64> = None;
    let mut tm: Option<u64> = None;
    let mut has_items: Option<bool> = None;

    let cnt: Option<usize> = None;
    if !is_dir {
        mime = Some(from_path(&name).first_or_octet_stream().to_string());
        ext = entry.extension().map(|ext| ext.to_string_lossy().to_string().to_lowercase());
    }

    if !meta_types.is_empty() {
        match entry.metadata() {
            Ok(meta) => {
                if is_dir {
                    if meta_types.contains(&MetaType::HasItems) {
                        has_items = Some(entry.has_children());
                    }
                } else {
                    if meta_types.contains(&MetaType::Size) {
                        size = Some(meta.len());
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
        name,
        is_dir,
        mime,
        ext,
        size,
        cnt,
        has_items,
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

fn sort_items(items: &mut Vec<Item>, ordering: Vec<OrdItem>) {
    items.sort_by(|a, b| {
        for ord in ordering.iter() {
            if OrderBy::Dir == ord.nm {
                match cmp_item(&b.is_dir, &a.is_dir, &ord.asc) {
                    Some(ord) => {
                        return ord
                    },
                    None => continue
                }
            } else if OrderBy::Name == ord.nm {
                let a_ord = a.name.to_lowercase();
                let b_ord = b.name.to_lowercase();
                match cmp_item(&a_ord, &b_ord, &ord.asc) {
                    Some(ord) => {
                        return ord
                    },
                    None => continue
                }
            } else if OrderBy::Ext == ord.nm && !a.is_dir {
                match (&a.ext, &b.ext) {
                    (Some(a), Some(b)) => {
                        let a_ord = a.to_lowercase();
                        let b_ord = b.to_lowercase();
                        match cmp_item(&a_ord, &b_ord, &ord.asc) {
                            Some(ord) => {
                                return ord
                            },
                            None => continue
                        }
                    }
                    _ => { continue }
                }
            } else if OrderBy::Mime == ord.nm && !a.is_dir {
                match (&a.mime, &b.mime) {
                    (Some(a), Some(b)) => {
                        let a_ord = a.to_lowercase();
                        let b_ord = b.to_lowercase();
                        match cmp_item(&a_ord, &b_ord, &ord.asc) {
                            Some(ord) => {
                                return ord
                            },
                            None => continue
                        }
                    }
                    _ => continue
                }
            } else if OrderBy::Size == ord.nm && a.size.ne(&b.size) {
                match (&a.size, &b.size) {
                    (Some(a), Some(b)) => {
                        match cmp_item(a, b, &ord.asc) {
                            Some(ord) => {
                                return ord
                            },
                            None => continue
                        }
                    }
                    _ => continue
                }
            }
        }
        if !ordering.iter().any(|o| o.nm == OrderBy::Name) {
            return a.name.cmp(&b.name)
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
        assert_eq!(api.dir(params).await.unwrap().base_dir, "C:");
    }

    #[tokio::test]
    async fn test_permissions() {
        let api = Api::default();
        let params = Params {
            path_str: String::from(r"C:\Windows\WinSxS"),
            ..Params::default()
        };
        assert!(api.dir(params).await.is_ok());
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
        assert!(api.dir(params).await.is_ok());

    }

    #[tokio::test]
    async fn test_get_folder() {
        let api = Api::default();
        let params = Params {
            path_str: String::from(r"C:\Windows\WinSxS"),
            // is_cache: false,
            ..Params::default()
        };
        match api.get_folder(params.clone()).await {
            Ok(json) => {
                println!("{}", json.len());
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
}