use std::cmp::Ordering;
use crate::models::{Item, MetaType, ApiError, OrderAsc, OrdItem, OrderBy};
use crate::system_time_ext::SystemTimeExt;
use std::path::{PathBuf};
use windows::{
    core::{
        PCWSTR
    },
    Win32::Foundation::{
        FILETIME, MAX_PATH, HANDLE,
    },
    Win32::Storage::FileSystem:: {
        FindNextFileW, FindClose, WIN32_FIND_DATAW,
        FindFirstFileExW, FIND_FIRST_EX_LARGE_FETCH,
        FindExInfoStandard, FindExSearchNameMatch,
        GetFullPathNameW,
    },
};
use std::ffi::OsStr;
use std::os::windows::ffi::{OsStrExt};
use mime_guess::from_path;
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
use windows::core::Error as WinError;


type Result<T> = std::result::Result<T, ApiError>;

pub struct FindHandle(HANDLE);
impl Drop for FindHandle {
    fn drop(&mut self) {
        match unsafe { FindClose(self.0) } {
            Ok(_) => { },
            Err(err) => { println!("{:?}", err) },
        }
    }
}

#[allow(dead_code)]
pub fn get_items(p: &str, meta_types: &Vec<MetaType>) -> Result<Vec<Item>> {
    let result = std::fs::read_dir(p)?.flatten()
        .filter_map(|entry| { get_item_data(entry.path(), &meta_types) }).collect();
    Ok(result)
}

pub fn get_items_win32(p: &str, meta_types: &Vec<MetaType>) -> Result<Vec<Item>> {
    let mut result = Vec::new();
    // let pattern = format!("{}/*", p);
    let pattern: Vec<u16> = OsStr::new(&format!("{}/*", p))
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };
    let handle = unsafe {
        FindFirstFileExW(
            PCWSTR::from_raw(pattern.as_ptr()),
            FindExInfoStandard,
            // FindExInfoBasic,
            &mut find_data as *mut _ as *mut _,
            FindExSearchNameMatch,
            None,
            FIND_FIRST_EX_LARGE_FETCH,
        )?
    };
    let _handle_guard = FindHandle(handle);
    loop {
        if let Some(item) = get_item_data_win32(&mut find_data, &meta_types) {
            result.push(item);
        }
        match unsafe { FindNextFileW(handle, &mut find_data) } {
            Ok(_handle) => {}
            Err(_error) => break,
        }
    }
    Ok(result)
}

#[allow(dead_code)]
fn get_item_win32(p: &str, meta_types: &Vec<MetaType>) -> Result<Option<Item>> {
    let pattern: Vec<u16> = OsStr::new(&format!("{}/*", p))
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };
    let handle = unsafe {
        FindFirstFileExW(
            PCWSTR::from_raw(pattern.as_ptr()),
            FindExInfoStandard,
            // FindExInfoBasic,
            &mut find_data as *mut _ as *mut _,
            FindExSearchNameMatch,
            None,
            FIND_FIRST_EX_LARGE_FETCH,
        )?
    };
    let _handle_guard = FindHandle(handle);
    Ok(get_item_data_win32(&mut find_data, meta_types))
}



// fn get_item_data(entry: DirEntry, meta_types: Option<Vec<MetaType>>) -> Option<Item> {
fn get_item_data(p: PathBuf, meta_types: &Vec<MetaType>) -> Option<Item> {
    // let nm = entry.file_name().to_string_lossy().to_string();
    // let p = PathBuf::from(nm.clone());
    let nm = p.file_name()?.to_string_lossy().to_string();
    let dir = p.is_dir();
    let mut ext = None;
    let mut sz = None;
    let mut tm = None;

        if meta_types.contains(&MetaType::Ext) {
            ext = p.extension()
                .map(|ext| ext.to_string_lossy().to_string().to_lowercase());
        }

    match p.metadata() {
        Ok(metadata) => {
            if meta_types.contains(&MetaType::Sz) {
                sz = Some(metadata.len());
            }
            if meta_types.contains(&MetaType::Tm) {
                tm = metadata.modified().map(|t|t.to_sec()).ok();
            }
        }
        Err(err) => {
            println!("{:?}", err);
        }
    };

    Some(Item {
        nm,
        dir,
        ext,
        tm,
        sz,
        ..Item::default()
    })
}


fn get_item_data_win32(find_data: &mut WIN32_FIND_DATAW, meta_types: &Vec<MetaType>) -> Option<Item> {
    let nm = String::from_utf16_lossy(
        &find_data.cFileName[..find_data.cFileName.iter().position(|&c| c == 0).unwrap_or(0)],
    );
    if nm.is_empty() || nm == "." || nm == ".." {
        return None
    }
    let dir = (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0) != 0;
    let mut ext = None;
    let mut sz = None;
    let mut tm = None;

    if meta_types.contains(&MetaType::Ext) {
        ext = get_extension(&nm).map(|x| x.to_string());
    }
    if meta_types.contains(&MetaType::Sz) {
        sz = Some(((find_data.nFileSizeHigh as u64) << 32) | find_data.nFileSizeLow as u64);
    }
    if meta_types.contains(&MetaType::Tm) {
        tm = Some(filetime_to_unix_time(find_data.ftLastWriteTime));
    }

    Some(Item {
        nm,
        dir,
        ext,
        tm,
        sz,
        ..Item::default()
    })
}

fn filetime_to_unix_time(filetime: FILETIME) -> u64 {
    let high = filetime.dwHighDateTime as u64;
    let low = filetime.dwLowDateTime as u64;
    let ticks = (high << 32) | low;
    const EPOCH_DIFF: u64 = 116444736000000000;
    let unix_time_100ns = ticks.saturating_sub(EPOCH_DIFF);
    unix_time_100ns / 10_000_000
}

pub fn get_extension(filename: &str) -> Option<&str> {
    filename.rsplit_once('.').and_then(|(_, ext)| {
        if ext.is_empty() {
            None
        } else {
            Some(ext)
        }
    })
}

#[allow(dead_code)]
pub fn get_full_path(path: &str) -> Result<String> {
    unsafe {

        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

        let mut buffer = vec![0u16; MAX_PATH as usize];

        let len = GetFullPathNameW(
            PCWSTR(wide_path.as_ptr()),
            Some(&mut buffer),
            None,
        );

        if len == 0 || len as usize > buffer.len() {
            return Err(ApiError::DirApi(WinError::from_win32()))
        }

        Ok(String::from_utf16_lossy(&buffer[..len as usize]))

    }
}

#[allow(dead_code)]
fn has_children (path: &str) -> std::io::Result<bool> {
    Ok(std::fs::read_dir(PathBuf::from(path))?.flatten().next().is_some())
}

#[allow(dead_code)]
fn has_children_win32(path: &str) -> Result<bool> {
    let pattern: Vec<u16> = OsStr::new(&format!("{path}\\*"))
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut find_data = unsafe { std::mem::zeroed::<WIN32_FIND_DATAW>() };

    let handle = unsafe {
        FindFirstFileExW(
            PCWSTR::from_raw(pattern.as_ptr()),
            FindExInfoStandard,
            // FindExInfoBasic,
            &mut find_data as *mut _ as *mut _,
            FindExSearchNameMatch,
            None,
            FIND_FIRST_EX_LARGE_FETCH,
        )?
    };

    let mut result = false;

    loop {
        let name = &find_data.cFileName;
        let len = name.iter().position(|&c| c == 0).unwrap_or(0);
        let filename = String::from_utf16_lossy(&name[..len]);

        if filename != "." && filename != ".." {
            result = true;
            break;
        }
        let next = unsafe { FindNextFileW(handle, &mut find_data) };
        if next.is_err() {
            break;
        }
    }

    Ok(result)
}

fn get_ext(nm: &str) ->  Option<String> {
    PathBuf::from(nm).extension().map(|ext| ext.to_string_lossy().to_string().to_lowercase())
}

fn get_mime_type(nm: &str) -> Option<String> {
    Some(from_path(&nm).first_or_octet_stream().to_string())
}

pub fn update_items(items: &mut Vec<Item>, meta_types: &Vec<MetaType>) {
    for item in items.iter_mut() {
        if meta_types.contains(&MetaType::Ext) {
            item.ext = get_ext(&item.nm)
        }
        if meta_types.contains(&MetaType::Mt) {
            item.mt = get_mime_type(&item.nm);
        }
    }
}

pub fn update_max_len_nm(items: &Vec<Item>) -> usize {
    let mut max_len: f32 = 0.0;
    for item in items {
        let a = item.nm.chars().count() as f32;
        let w = item.nm.chars().filter(|&c| is_fullwidth(c)).count() as f32;
        let r = (a-w) + w*1.5;
        if r > max_len {
            max_len = r;
        }
    }
    max_len.round() as usize
}

fn is_fullwidth(c: char) -> bool {
    match c as u32 {
        0x1100..=0x115F   // Hangul Jamo
        | 0x2E80..=0xA4CF // CJK, Kana, etc.
        | 0xAC00..=0xD7A3 // Hangul Syllables
        | 0xF900..=0xFAFF // CJK Compatibility
        | 0xFE10..=0xFE19
        | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60 // Fullwidth ASCII
        | 0xFFE0..=0xFFE6 => true,
        _ => false,
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

fn cmp_str_item(a: &String, b: &String, asc: &OrderAsc) -> Option<Ordering> {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    cmp_item(&a, &b, asc)
}

fn cmp_opt_str_item(a: &Option<String>, b: &Option<String>, asc: &OrderAsc) -> Option<Ordering> {
    match (a, b) {
        (Some(a), Some(b)) => cmp_str_item(a, b, asc),
        _ => None
    }
}

fn cmp_opt_item<T: Ord>(a: &Option<T>, b: &Option<T>, asc: &OrderAsc) -> Option<Ordering> {
    match (a, b) {
        (Some(a), Some(b)) => cmp_item(a, b, asc),
        _ => None
    }
}



pub fn sort_items(items: &mut Vec<Item>, ordering: &Vec<OrdItem>) {
    items.sort_by(|a, b| {
        for ord in ordering.iter() {
            let res = match ord.nm {
                OrderBy::Dir => cmp_item(&b.dir, &a.dir, &ord.asc),
                OrderBy::Nm => cmp_str_item(&a.nm, &b.nm, &ord.asc),
                OrderBy::Ext if !a.dir => cmp_opt_str_item(&a.ext, &b.ext, &ord.asc),
                OrderBy::Mt if !a.dir => cmp_opt_str_item(&a.mt, &b.mt, &ord.asc),
                OrderBy::Sz if a.sz.ne(&b.sz)  => cmp_opt_item(&a.sz, &b.sz, &ord.asc),
                _ => None,
            };
            if let Some(ord) = res {
                return ord;
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
    use super::*;
    #[test]
    fn test_speed() {
        // let base_dir= r"C:\Windows\WinSxS";
        let base_dir= r"C:\";
        let meta_types = vec![MetaType::Sz, MetaType::Tm, MetaType::Ext, MetaType::Mt];
        let ordering = vec![
            OrdItem{nm: OrderBy::Nm, asc: OrderAsc::Asc},
            OrdItem{nm: OrderBy::Tm, asc: OrderAsc::Asc},
        ];
        let mut items = get_items_win32(base_dir, &meta_types).unwrap();
        update_items(&mut items, &meta_types);
        sort_items(&mut items, &ordering);
    }


    #[test]
    fn test_has_children_win32() {
        // let base_dir= r"C:\Windows\WinSxS";
        let base_dir= r"C:\";
        let v: Vec<_> = get_items(base_dir, &vec![]).into_iter().flatten().map(|item| {
            let path = format!(r"{}\{}", base_dir, item.nm);
            has_children_win32(path.as_str()).unwrap_or_else(|_| false)
        }).collect();
        println!("{}", v.len());
    }

    #[test]
    fn test_has_children() {
        // let base_dir= r"C:\Windows\WinSxS";
        let base_dir= r"C:\";
        let v: Vec<_> = get_items_win32(base_dir, &vec![]).into_iter().flatten().map(|item| {
            let path = format!(r"{}\{}", base_dir, item.nm);
            has_children(path.as_str()).unwrap_or_else(|_| false)
        }).collect();
        println!("{}", v.len());
    }

    #[test]
    fn test_get_paths_win32() {
        // let s = r"C:\Windows\WinSxS";
        // let s = r"C://MSOCache";
        let s = r"C:\";

        match get_items_win32(s, &vec![]) {
            Ok(paths) => println!("{:?}", paths.len()),
            Err(error) => println!("{:?}", error),
        }

        // assert!(api.get_items(PathBuf::from(s)).await.is_err());
        // assert!(api.get_paths(s).is_ok());
    }


    #[test]
    fn test_get_paths() {
        // let s = r"C:\Windows\WinSxS";
        // let s = r"C://MSOCache";
        let s = r"C:\";

        match get_items(s, &vec![]) {
            Ok(paths) => println!("{:?}", paths.len()),
            Err(error) => println!("{:?}", error),
        }

        // assert!(api.get_items(PathBuf::from(s)).await.is_err());
        // assert!(api.get_paths(s).is_ok());
    }

    #[test]
    fn test_get_full_path() {
        let s = r".";
        println!( "{:?}", get_full_path(s).unwrap());
    }

}