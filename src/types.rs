use std::{borrow::Cow, collections::HashMap, ops::RangeInclusive};

use crate::entity::EntityType;

pub type Byte = u8;
pub type ByteList = Vec<Byte>;
pub type Bytes = &'static [Byte];
pub type EntityCharBytes = HashMap<char, Bytes>;
pub type BytesCharEntity = HashMap<Bytes, char>;
pub type AnyhowResult<T> = anyhow::Result<T>;
pub type StringResult = AnyhowResult<String>;
pub type CharListResult = AnyhowResult<Vec<char>>;
pub type CodeRange = RangeInclusive<usize>;
pub type CodeRangeTuple = (usize, usize);
pub type IterDataItem<'a> = (&'a Byte, Option<CodeRangeTuple>);
pub type EncodeFilterReturnData = (bool, Option<(EntityType, Cow<'static, [Byte]>)>);
