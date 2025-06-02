#![deny(clippy::all)]

mod models;
mod path_ext;
mod system_time_ext;
mod api;

// use std::future::Future;
use napi_derive::napi;
use napi::{Error as NApiError};
use serde::{Serialize, Deserialize};
use crate::api::get_instance;
use crate::models::{OrdItem, OrderAsc, OrderBy, MetaType, OptParams, Params, ApiError, TextContent};



#[napi]
pub struct FolderApi;

#[napi]
impl FolderApi {
  #[napi(constructor)]
  pub fn new() -> Self {
    FolderApi
  }

  #[napi]
  pub async fn read_text(&self, path_str: String) -> Result<String, NApiError> {
    let text_content: TextContent = get_instance().read_txt(&path_str).await.map_err(Into::<NApiError>::into)?;
    self.from_obj(&text_content, false).map_err(Into::<NApiError>::into)      
  }

  #[napi]
  pub async fn read_folder(&self, json_params: String) -> Result<String, NApiError> {
    let params: OptParams = self.from_str(json_params.as_str()).map_err(Into::<NApiError>::into)?;

    let new_params = Params {
      meta_types: params.meta_types.unwrap_or(vec![MetaType::Sz, MetaType::Tm, MetaType::Has]),
      ordering: params.ordering.unwrap_or(vec![OrdItem { nm: OrderBy::Dir, asc: OrderAsc::Asc }, OrdItem { nm: OrderBy::Nm, asc: OrderAsc::Asc }]),
      is_pretty: params.is_pretty.unwrap_or(false),
      path_str: params.path_str.unwrap_or(String::from(".")),
      cache_nm: params.cache_nm,
      skip_n: params.skip_n,
      take_n: params.take_n,
    };
    let folder = get_instance().get_folder(&new_params).await.map_err(Into::<NApiError>::into)?;
    
    self.from_obj(&folder, new_params.is_pretty).map_err(Into::<NApiError>::into)
  }

  ///
  /// set state
  ///
  /// # arg
  /// - key
  /// - opt_val: if `None` then delete cache
  ///
  /// # Examples
  /// ```
  /// set_stat("key".to_string(), Some("val".to_string())
  /// set_state("key".to_string(), None)
  /// ```
  #[napi]
  pub async fn set_state(&self, key: String, val: Option<String>) -> Result<Option<String>, NApiError> {
    let val = get_instance().set_state(key, val).await?;
    println!("set_state: {:?}", val);
    Ok(val)
  }

  ///
  /// get state
  ///
  /// # arg
  /// - key
  /// - default_val: If the key does not exists in the cache, inserts the default value and return it.
  #[napi]
  pub async fn get_state(&self, key: String, default_val: Option<String>) -> Result<Option<String>, NApiError> {
    let val = get_instance().get_state(&key, default_val).await?;
    println!("get_state: {:?}", val);
    Ok(val)
  }

  fn from_str<'a, T> (&self, json_str: &'a str) -> Result<T, ApiError>
  where
      T: Deserialize<'a> {
    Ok(serde_json::from_str(json_str)?)
  }

  fn from_obj<'a, T> (&self, obj: &T, is_pretty: bool) -> Result<String, ApiError>
    where T: ?Sized + Serialize, {
    
    let fn_json = if is_pretty {
      serde_json::to_string_pretty
    } else {
      serde_json::to_string
    };
    Ok(fn_json(obj)?)
  }

}


// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   #[tokio::test]
//   async fn test_abc() {
//     let folderApi = FolderApi{};
//     folderApi.
//
//   }
// }