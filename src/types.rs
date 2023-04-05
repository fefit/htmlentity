use std::{collections::HashMap, ops::RangeInclusive};

pub type Byte = u8;
pub type ByteList = Vec<Byte>;
pub type Bytes = &'static [Byte];
pub type EntityCharBytes = HashMap<char, Bytes>;
pub type BytesCharEntity = HashMap<Bytes, char>;
pub type AnyhowResult<T> = anyhow::Result<T>;
pub type StringResult = AnyhowResult<String>;
pub type CharListResult = AnyhowResult<Vec<char>>;
pub type CodeRange = RangeInclusive<usize>;
pub type IterDataItem<'a> = (&'a Byte, Option<(usize, usize)>);
