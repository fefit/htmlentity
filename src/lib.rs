//! # htmlentity crate
//!
//! A library for encoding and decoding HTML entities.
//!
//! # Examples
//!
//! ```
//! use htmlentity::entity::{
//!     encode, decode,
//!     EncodeType, CharacterSet, ICodedDataTrait
//! };
//! use htmlentity::types::{ AnyhowResult, Byte };
//! # fn main() -> AnyhowResult<()> {
//! let html = "<div name='htmlentity'>Hello!世界!</div>";
//! let html_after_encoded = "&lt;div name='htmlentity'&gt;Hello!&#x4e16;&#x754c;!&lt;/div&gt;";
//! // encode
//! let encoded_data = encode(
//!     html.as_bytes(),
//!     &EncodeType::NamedOrHex,
//!     &CharacterSet::HtmlAndNonASCII
//! );
//! // encoded data to bytes
//! assert_eq!(
//!     encoded_data.to_bytes(),
//!     html_after_encoded.as_bytes()
//! );
//! // encoded data to string
//! assert_eq!(
//!     encoded_data.to_string()?,
//!     String::from(html_after_encoded)
//! );
//! // encoded data to chars
//! assert_eq!(
//!     encoded_data.to_chars()?,
//!     String::from(html_after_encoded).chars().collect::<Vec<char>>()
//! );
//! // decode
//! let bytes = encoded_data
//!     .into_iter()
//!     .map(|(byte, _)| *byte)
//!     .collect::<Vec<Byte>>();
//! let decoded_data = decode(&bytes);
//! // decoded data to bytes
//! assert_eq!(
//!     decoded_data.to_bytes(),
//!     html.as_bytes()
//! );
//! // decoded data to string
//! assert_eq!(
//!     decoded_data.to_string()?,
//!     String::from(html)
//! );
//! // decoded data to chars
//! assert_eq!(
//!     decoded_data.to_chars()?,
//!     String::from(html).chars().collect::<Vec<char>>()
//! );
//! # Ok(())
//! # }
//! ```
/// The html entities data.
pub mod data;
/// The library main module.
pub mod entity;
/// The library's types.
pub mod types;
