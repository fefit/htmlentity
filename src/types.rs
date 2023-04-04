use std::{borrow::Cow, collections::HashMap};

use crate::entity::EncodeType;

pub type Byte = u8;
pub type ByteList = Vec<Byte>;
pub type Bytes = &'static [Byte];
pub type EntityCharBytes = HashMap<char, Bytes>;
pub type BytesCharEntity = HashMap<Bytes, char>;
