#![deny(clippy::all)]

mod models;
mod path_ext;
mod system_time_ext;
mod api;

// use std::future::Future;
use napi_derive::napi;
// use napi::bindgen_prelude::{Result, Error, Status};
use napi::{Error as NApiError};
use serde::{Serialize, Deserialize};
use crate::api::get_instance;
use crate::models::{OrdItem, OrderAsc, OrderBy, MetaType, OptParams, Params, StateParams, ApiError};



#[napi]
pub struct FolderApi;

#[napi]
impl FolderApi {
  #[napi(constructor)]
  pub fn new() -> Self {
    FolderApi
  }

  // fn handle_json_call<T, U, E, F, Fut>(&self, json: &str, func: F) -> Result<String, ApiError>
  // where
  //     T: for<'de> Deserialize<'de>,  // OptParams
  //     U: Serialize,  // Folder
  //     // F: FnOnce(&T) -> Result<U>,  // func(&OptParams) -> Folder
  //     F: FnOnce(&T) -> Fut,
  //     Fut: Future<Output = Result<U>> + Serialize,
  //     E: From<Error>,
  // {
  //   let input: T = self.from_json_string(json)?;
  //   let output = func(&input);
  //   Ok(self.to_json_string(&output, false)?)
  // }


  #[napi]
  pub async fn read_folder(&self, json_params: String) -> Result<String, NApiError> {
    let params: OptParams = self.from_str(json_params.as_str()).map_err(Into::<NApiError>::into)?;

    // self.handle_json_call::<OptParams, Folder, _, _, _>(json_params.as_str(), |params| {
    //   let new_params = Params {
    //     meta_types: params.meta_types.unwrap_or(vec![MetaType::Size, MetaType::Tm, MetaType::HasItems]),
    //     ordering: params.ordering.unwrap_or(vec![OrdItem { nm: OrderBy::Dir, asc: OrderAsc::Asc }, OrdItem { nm: OrderBy::Name, asc: OrderAsc::Asc }]),
    //     is_pretty: params.is_pretty.unwrap_or(false),
    //     is_cache: params.is_cache.unwrap_or(true),
    //     path_str: params.path_str.unwrap_or(String::from(".")),
    //     skip_n: params.skip_n,
    //     take_n: params.take_n,
    //   };
    //   get_instance().get_folder(&new_params)
    //   // get_instance().get_folder(&new_params).await.map_err(|e| {
    //   //   println!("err: {:?}", e);
    //   //   Error::new(Status::Unknown, e.to_string())
    //   // })
    // })

    let new_params = Params {
      meta_types: params.meta_types.unwrap_or(vec![MetaType::Size, MetaType::Tm, MetaType::HasItems]),
      ordering: params.ordering.unwrap_or(vec![OrdItem { nm: OrderBy::Dir, asc: OrderAsc::Asc }, OrdItem { nm: OrderBy::Name, asc: OrderAsc::Asc }]),
      is_pretty: params.is_pretty.unwrap_or(false),
      is_cache: params.is_cache.unwrap_or(true),
      path_str: params.path_str.unwrap_or(String::from(".")),
      skip_n: params.skip_n,
      take_n: params.take_n,
    };

    let folder = get_instance().get_folder(&new_params).await.map_err(Into::<NApiError>::into)?;
    self.from_obj(&folder, new_params.is_pretty).map_err(Into::<NApiError>::into)
  }

  #[napi]
  pub async fn get_state(&self, json_params: String) -> Result<String, NApiError> {
    let params: StateParams = self.from_str(json_params.as_str())?;

    let state_params = get_instance().get_state(&params.key).await?;

    self.from_obj(&state_params, false).map_err(Into::<NApiError>::into)
  }

  #[napi]
  pub async fn set_state(&self, json_params: String) -> Result<String, NApiError> {
    let params: StateParams = self.from_str(json_params.as_str())?;

    let state_params = get_instance().set_state(params.key, params.val).await?;

    self.from_obj(&state_params, false).map_err(Into::<NApiError>::into)
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


