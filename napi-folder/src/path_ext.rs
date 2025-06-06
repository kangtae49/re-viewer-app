use std::path::Component::RootDir;
use std::path::Path;

pub trait PathExt {
    fn is_root(&self) -> bool;
    // fn has_children(&self) -> bool;
    // fn get_cnt(&self) -> Option<usize>;
}
impl PathExt for Path {
    fn is_root(&self) -> bool {
        match self.components().last() {
            Some(RootDir) => true,
            _ => false,
        }
    }

    // fn has_children(&self) -> bool {
    //     match self.read_dir() {
    //         Ok(mut entry) => {
    //             entry.next().is_some()
    //         }
    //         Err(_) => {
    //             false
    //         }
    //     }
    // }

    // fn get_cnt(&self) -> Option<usize> {
    //     match self.read_dir() {
    //         Ok(entry) => {
    //             Some(entry.count())
    //         }
    //         Err(err) => {
    //             println!("err: {:?}", err);
    //             None
    //         }
    //     }
    // }

}