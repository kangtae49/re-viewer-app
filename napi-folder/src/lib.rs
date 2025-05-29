#![deny(clippy::all)]

mod models;
mod path_ext;
mod system_time_ext;
mod api;

use napi_derive::napi;
// use napi::bindgen_prelude::*;
use napi::bindgen_prelude::{Result, Error, Status};
use crate::api::get_instance;
use crate::models::{OrdItem, OrderAsc, OrderBy, MetaType, OptParams, Params};


// #[napi(object)]
// pub struct NodeOrdItem {
//   pub nm: String,
//   pub asc: String,
// }

#[napi]
pub struct FolderApi;

#[napi]
impl FolderApi {
  #[napi(constructor)]
  pub fn new() -> Self {
    FolderApi
  }

  #[napi]
  // pub async fn read_folder(&self, path_str: String, skip_n: Option<u32>, take_n: Option<u32>, order: Option<Vec<NodeOrdItem>>) -> Result<String> {

  pub async fn read_folder(&self, json_params: String) -> Result<String> {
    let opt_params: OptParams = match serde_json::from_str(json_params.as_str()) {
      Ok(p) => p,
      Err(e) => {
        println!("{:?}", e);
        return Err(Error::new(Status::InvalidArg, e.to_string()));
      }
    };

    let new_params = Params {
      meta_types: opt_params.meta_types.unwrap_or(vec![MetaType::Size, MetaType::Tm, MetaType::HasItems]),
      ordering: opt_params.ordering.unwrap_or(vec![OrdItem { nm: OrderBy::Dir, asc: OrderAsc::Asc }, OrdItem { nm: OrderBy::Name, asc: OrderAsc::Asc }]),
      is_pretty: opt_params.is_pretty.unwrap_or(false),
      is_cache: opt_params.is_cache.unwrap_or(true),
      path_str: opt_params.path_str.unwrap_or(String::from(".")),
      skip_n: opt_params.skip_n,
      take_n: opt_params.take_n,
    };
    let s = get_instance().get_folder(new_params).await?;
    Ok(s)
  }
}

// 
// fn to_ord_item(a: &NodeOrdItem) -> OrdItem {
//   OrdItem {
//     nm: match a.nm.to_lowercase().as_str() {
//       "dir" => OrderBy::Dir,
//       "name" => OrderBy::Name,
//       "size" => OrderBy::Size,
//       "tm" => OrderBy::Tm,
//       "mime" => OrderBy::Mime,
//       "ext" => OrderBy::Ext,
//       _ => OrderBy::Dir,
//     },
//     asc: match a.asc.to_lowercase().as_str() {
//       "asc" => OrderAsc::Asc,
//       "desc" => OrderAsc::Desc,
//       _ => OrderAsc::Asc,
//     }
//   }
// }
