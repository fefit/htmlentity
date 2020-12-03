#![cfg(target_arch = "wasm32")]
use crate::entity::{decode as r_decode, encode as r_encode, EncodeType, EntitySet};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "IString")]
  pub type IString;
}

#[wasm_bindgen(typescript_custom_section)]
const IJS_STRING: &'static str = r#"
export type IString = string;
"#;

#[wasm_bindgen]
pub fn encode(
  content: &str,
  entities: Option<EntitySet>,
  encode_type: Option<EncodeType>,
) -> IString {
  let result = r_encode(
    content,
    entities.unwrap_or_default(),
    encode_type.unwrap_or_default(),
  );
  JsValue::from_str(&result).into()
}

#[wasm_bindgen]
pub fn decode(content: &str) -> IString {
  JsValue::from_str(&r_decode(content)).into()
}
