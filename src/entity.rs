use crate::{
  data::{ENTITIES, FIRST_LETTER_POSITION, LETTER_ORDERED_ENTITIES},
  types::{
    AnyhowResult, Byte, ByteList, BytesCharEntity, CharListResult, CodeRange, CodeRangeTuple,
    EncodeFilterReturnData, EntityCharBytes, IterDataItem, StringResult,
  },
};

use lazy_static::lazy_static;
use std::{borrow::Cow, char, cmp::Ordering, collections::HashMap};
use thiserror::Error;

lazy_static! {
  // html bytes
  static ref HTML_BYTES: EntityCharBytes  = {
    let mut map: EntityCharBytes  = HashMap::with_capacity(3);
    map.insert('>', b"gt");
    map.insert('<', b"lt");
    map.insert('&', b"amp");
    map
  };
  // special bytes
  static ref SPECIAL_BYTES: EntityCharBytes  = {
    let mut map: EntityCharBytes  = HashMap::with_capacity(5);
    map.insert('"', b"quot");
    map.insert('\'', b"apos");
    for (k, v) in HTML_BYTES.iter(){
        map.insert(*k, *v);
    }
    map
  };
  // normal name entity
  static ref NORMAL_NAME_ENTITY_BYTE: BytesCharEntity = {
    let mut map: BytesCharEntity = HashMap::with_capacity(10);
    map.insert(b"lt", '<');
    map.insert(b"LT", '<');
    map.insert(b"gt", '>');
    map.insert(b"GT", '>');
    map.insert(b"amp", '&');
    map.insert(b"AMP", '&');
    map.insert(b"quot", '"');
    map.insert(b"QUOT", '"');
    map.insert(b"apos", '\'');
    map.insert(b"nbsp", 0xa0 as char);
    map
  };
}

#[derive(Error, Debug)]
pub enum HtmlEntityError {
  #[error("Decode error: {0}")]
  Decode(String),
  #[error("Encode error: {0}")]
  Encode(String),
}

#[inline]
fn char_to_utf8_bytes(ch: char) -> ByteList {
  let len = ch.len_utf8();
  let mut bytes: ByteList = vec![];
  bytes.resize(len, 0);
  ch.encode_utf8(&mut bytes);
  bytes
}

#[inline]
fn tr_chars_to_utf8_bytes(chars: &[char]) -> Option<ByteList> {
  let mut bytes: ByteList = vec![];
  for ch in chars {
    if ch.len_utf8() == 1 {
      bytes.push(*ch as Byte);
      continue;
    }
    return None;
  }
  Some(bytes)
}

#[inline]
fn numbers_to_char(bytes: &[Byte], radix: u32) -> AnyhowResult<char> {
  if !bytes.is_empty() {
    // '&#;' '&#x;'
    let num = std::str::from_utf8(bytes)?;
    let char_code = i64::from_str_radix(num, radix)?;
    return std::char::from_u32(char_code as u32).ok_or(
      HtmlEntityError::Decode(format!(
        "The html entity number '&{}{};' is not a valid encoded character.",
        if radix == 16 { "#" } else { "" },
        num
      ))
      .into(),
    );
  }
  Err(HtmlEntityError::Decode(String::from("Html entity number cannot be empty.")).into())
}

enum Utf8ParsedData {
  Correct(char),
  Wrong(&'static str),
}

#[inline]
fn loop_utf8_bytes(
  bytes: &[Byte],
  mut handle: impl FnMut(Utf8ParsedData, CodeRangeTuple) -> AnyhowResult<()>,
) -> AnyhowResult<()> {
  let mut next_count = 0;
  let mut ch: u32 = 0;
  let mut start_index: usize = 0;
  for (index, byte) in bytes.iter().enumerate() {
    match next_count {
      0 => {
        start_index = index;
        if (byte >> 7) == 0 {
          let _ = handle(Utf8ParsedData::Correct(*byte as char), (start_index, index));
        } else {
          let mut head = byte >> 3;
          if head == 0b11110 {
            next_count = 3;
            ch = ((byte & 0b111) as u32) << (next_count * 6);
          } else {
            head >>= 1;
            if head == 0b1110 {
              next_count = 2;
              ch = ((byte & 0b1111) as u32) << (next_count * 6);
            } else {
              head >>= 1;
              if head == 0b110 {
                next_count = 1;
                ch = ((byte & 0b11111) as u32) << (next_count * 6);
              } else {
                // wrong utf8 byte
                next_count = 0;
                handle(
                  Utf8ParsedData::Wrong("Illegal utf8 encoded bytes"),
                  (start_index, index),
                )?;
              }
            }
          }
        }
      }
      1 | 2 | 3 => {
        if (byte >> 6) == 0b10 {
          next_count -= 1;
          ch += ((byte & 0b111111) as u32) << (next_count * 6);
          if next_count == 0 {
            if let Some(ch) = char::from_u32(ch) {
              let _ = handle(Utf8ParsedData::Correct(ch), (start_index, index));
            } else {
              handle(
                Utf8ParsedData::Wrong("Illegal encoding utf8 character."),
                (start_index, index),
              )?;
            }
          }
        } else {
          next_count = 0;
          // wrong utf8
          handle(
            Utf8ParsedData::Wrong("Illegal utf8 encoded bytes."),
            (start_index, index),
          )?;
        }
      }
      // unreachable feature
      _ => unreachable!(),
    }
  }
  Ok(())
}

#[inline]
fn bytes_to_chars(bytes: &[Byte], data: &mut Vec<char>) -> AnyhowResult<()> {
  loop_utf8_bytes(bytes, |result, _| match result {
    Utf8ParsedData::Correct(ch) => {
      data.push(ch);
      Ok(())
    }
    Utf8ParsedData::Wrong(message) => Err(HtmlEntityError::Decode(String::from(message)).into()),
  })
}

#[inline]
fn call_into_char_list_trait<T>(
  bytes: &[Byte],
  entities: &[(CodeRange, T)],
  handle: impl Fn(&T, &mut Vec<char>),
) -> CharListResult {
  let total = bytes.len();
  let mut result: Vec<char> = Vec::with_capacity(total / 2);
  if entities.is_empty() {
    bytes_to_chars(bytes, &mut result)?;
    return Ok(result);
  }
  let mut index = 0;
  for (range, item) in entities {
    let start_index = *range.start();
    let end_index = *range.end();
    if index < start_index {
      bytes_to_chars(&bytes[index..start_index], &mut result)?;
    }
    handle(item, &mut result);
    index = end_index + 1;
  }
  if index < total {
    bytes_to_chars(&bytes[index..], &mut result)?;
  }
  Ok(result)
}

#[inline]
fn call_into_string_trait<T>(
  bytes: &[Byte],
  entities: &[(CodeRange, T)],
  handle: impl Fn(&T, &mut String),
) -> StringResult {
  if entities.is_empty() {
    let code = std::str::from_utf8(bytes)?;
    return Ok(String::from(code));
  }
  let total = bytes.len();
  let mut result = String::with_capacity(total);
  let mut index = 0;
  for (range, item) in entities {
    let start_index = *range.start();
    let end_index = *range.end();
    if index < start_index {
      let code = std::str::from_utf8(&bytes[index..start_index])?;
      result.push_str(code);
    }
    handle(item, &mut result);
    index = end_index + 1;
  }
  if index < total {
    let code = std::str::from_utf8(&bytes[index..])?;
    result.push_str(code);
  }
  Ok(result)
}

#[inline]
fn gen_into_iter<'a, T: IBytesTrait>(
  bytes: &'a [Byte],
  entities: &'a [(CodeRange, T)],
) -> DataIter<'a, T> {
  let total_entities = entities.len();
  let only_bytes = total_entities == 0;
  let byte_index_of_next_entity = if only_bytes {
    None
  } else {
    Some(*entities[0].0.start())
  };
  DataIter {
    byte_index: 0,
    total_bytes: bytes.len(),
    entity_index: 0,
    total_entities,
    only_bytes,
    byte_index_of_next_entity,
    byte_index_entity_looped: 0,
    bytes,
    entities,
  }
}

#[inline]
fn call_trait_method_bytes_len<T: IBytesTrait>(
  bytes: &[Byte],
  entities: &[(CodeRange, T)],
) -> usize {
  if entities.is_empty() {
    return bytes.len();
  }
  let mut start_index = 0;
  let mut len: usize = 0;
  for (range, entity) in entities {
    len += range.start() - start_index;
    len += entity.bytes_len();
    start_index = *range.end() + 1;
  }
  len += bytes.len() - start_index;
  len
}

#[inline]
fn call_trait_method_byte<'a, T: IBytesTrait>(
  bytes: &'a [Byte],
  entities: &'a [(CodeRange, T)],
  mut index: usize,
) -> Option<&'a Byte> {
  if entities.is_empty() {
    return bytes.get(index);
  }
  let mut prev_start_byte_index: usize = 0;
  for (range, entity) in entities {
    let start_byte_index = *range.start();
    let cur_index = prev_start_byte_index + index;
    if cur_index < start_byte_index {
      // in the bytes between 'start_index' to 'start_byte_index'
      return bytes.get(cur_index);
    }
    let entity_len = entity.bytes_len();
    let cur_entity_index = cur_index - start_byte_index;
    if cur_entity_index < entity_len {
      // in entity
      return entity.byte(cur_entity_index);
    }
    index = cur_entity_index - entity_len;
    prev_start_byte_index = range.end() + 1;
  }
  bytes.get(prev_start_byte_index + index)
}

/// DecodedData, impl the ICodedDataTrait and IBytesTrait and IntoIterator.
#[derive(Debug)]
pub struct DecodedData<'b> {
  inner_bytes: Cow<'b, [Byte]>,
  entities: Vec<(CodeRange, (char, ByteList))>,
  errors: Vec<(CodeRange, anyhow::Error)>,
}

impl<'b> ICodedDataTrait for DecodedData<'b> {}

impl<'b> IBytesTrait for DecodedData<'b> {
  // bytes len
  fn bytes_len(&self) -> usize {
    call_trait_method_bytes_len(&self.inner_bytes, &self.entities)
  }
  // byte
  fn byte(&self, index: usize) -> Option<&Byte> {
    call_trait_method_byte(&self.inner_bytes, &self.entities, index)
  }
}

impl<'b> DecodedData<'b> {
  // detect if has errors
  pub fn is_ok(&self) -> bool {
    self.errors.is_empty()
  }
  // get errors
  pub fn get_errors(&self) -> &[(CodeRange, anyhow::Error)] {
    &self.errors
  }
  // entity count
  pub fn entity_count(&self) -> usize {
    self.entities.len()
  }
  // to owned
  pub fn to_owned(&mut self) {
    if self.entities.is_empty() {
      self.inner_bytes = self.inner_bytes.clone();
    } else {
      let bytes = self.to_bytes();
      self.inner_bytes = Cow::Owned(bytes);
      self.entities.clear();
    }
  }
  // into bytes
  pub fn into_bytes(self) -> ByteList {
    if self.entities.is_empty() {
      return self.inner_bytes.into_owned();
    }
    self.to_bytes()
  }
  // get bytes with cow
  pub fn bytes(&self) -> Cow<'b, [Byte]> {
    if self.entities.is_empty() {
      return self.inner_bytes.clone();
    }
    return Cow::Owned(self.to_bytes());
  }
}

pub trait IBytesTrait {
  fn byte(&self, index: usize) -> Option<&Byte>;
  fn bytes_len(&self) -> usize;
}

impl IBytesTrait for (char, ByteList) {
  fn byte(&self, index: usize) -> Option<&Byte> {
    self.1.get(index)
  }
  fn bytes_len(&self) -> usize {
    self.1.len()
  }
}

impl IBytesTrait for CharEntity {
  fn byte(&self, index: usize) -> Option<&Byte> {
    let prefix_len = self.prefix_len();
    if index > prefix_len {
      // from entity data or
      let cur_index = index - prefix_len - 1;
      return match cur_index.cmp(&self.entity_data.len()) {
        Ordering::Less => self.entity_data.get(cur_index),
        Ordering::Equal => Some(&b';'),
        Ordering::Greater => None,
      };
    } else if index == 0 {
      // the first byte
      return Some(&b'&');
    } else {
      // the next prefix bytes
      match prefix_len {
        1 => Some(&b'#'),
        2 => {
          if index == 1 {
            return Some(&b'#');
          }
          Some(&b'x')
        }
        _ => unreachable!(),
      }
    }
  }
  fn bytes_len(&self) -> usize {
    let prefix_len = self.prefix_len();
    // '&;' => 2 '#'|'#x' => prefix_len
    2 + prefix_len + self.entity_data.len()
  }
}

/// ICodedDataTrait
pub trait ICodedDataTrait
where
  for<'a> &'a Self: Into<StringResult> + Into<ByteList> + Into<CharListResult>,
{
  // to string
  fn to_string(&self) -> StringResult {
    self.into()
  }
  // to byptes
  fn to_bytes(&self) -> ByteList {
    self.into()
  }
  // to char list
  fn to_chars(&self) -> CharListResult {
    self.into()
  }
}

pub struct DataIter<'a, T: IBytesTrait> {
  only_bytes: bool,
  byte_index: usize,
  total_bytes: usize,
  entity_index: usize,
  total_entities: usize,
  byte_index_entity_looped: usize,
  byte_index_of_next_entity: Option<usize>,
  bytes: &'a [Byte],
  entities: &'a [(CodeRange, T)],
}

impl<'a, T: IBytesTrait> Iterator for DataIter<'a, T> {
  type Item = IterDataItem<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    let cur_byte_index = self.byte_index;
    if cur_byte_index < self.total_bytes {
      if self.only_bytes {
        // all the entities have been looped
        // or no entities exist
        self.byte_index += 1;
        return Some((&self.bytes[cur_byte_index], None));
      }
      let looped_index = self.byte_index_entity_looped;
      if looped_index == 0 {
        // if only_bytes = false
        // the next entity byte index must always has a value
        let next_index = self.byte_index_of_next_entity.unwrap();
        // when the byte index equal to next entity start index
        // should loop the entity instead of the bytes
        if cur_byte_index != next_index {
          self.byte_index += 1;
          return Some((&self.bytes[cur_byte_index], None));
        }
        // otherwise should loop the entity bytes
      }
      let cur_entity = &self.entities[self.entity_index];
      let cur_byte = &cur_entity
        .1
        .byte(looped_index)
        .expect("The 'byte' method must use a correct 'index' parameter.");
      let entity_position = Some((self.entity_index, looped_index));
      if looped_index == cur_entity.1.bytes_len() - 1 {
        // end the cur_entity_bytes
        self.byte_index_entity_looped = 0;
        self.entity_index += 1;
        // reset the byte index to the next of entity end index
        self.byte_index = cur_entity.0.end() + 1;
        // judge if entities have all looped
        if self.entity_index < self.total_entities {
          self.byte_index_of_next_entity = Some(*self.entities[self.entity_index].0.start());
        } else {
          // now only bytes left
          self.only_bytes = true;
        }
      } else {
        self.byte_index_entity_looped += 1;
      }
      return Some((cur_byte, entity_position));
    }
    None
  }
}

impl<'a> IntoIterator for &'a DecodedData<'a> {
  type Item = IterDataItem<'a>;
  type IntoIter = DataIter<'a, (char, ByteList)>;
  fn into_iter(self) -> Self::IntoIter {
    gen_into_iter(&self.inner_bytes, &self.entities)
  }
}

impl<'a> IntoIterator for &'a EncodedData<'a> {
  type Item = IterDataItem<'a>;
  type IntoIter = DataIter<'a, CharEntity>;
  fn into_iter(self) -> Self::IntoIter {
    gen_into_iter(&self.inner_bytes, &self.entities)
  }
}
/**
 * impl decode data to string
 *  
 *
 */
impl<'b> From<&DecodedData<'b>> for StringResult {
  fn from(value: &DecodedData<'b>) -> Self {
    call_into_string_trait(&value.inner_bytes, &value.entities, |&(ch, _), result| {
      result.push(ch)
    })
  }
}

impl<'b> From<DecodedData<'b>> for StringResult {
  fn from(value: DecodedData<'b>) -> Self {
    (&value).into()
  }
}

/**
 * impl decode data into vec bytes
 */
impl<'b> From<&DecodedData<'b>> for ByteList {
  fn from(value: &DecodedData<'b>) -> Self {
    value
      .into_iter()
      .map(|(byte, _)| *byte)
      .collect::<ByteList>()
  }
}
// easy to call `decode(data).into()`
impl<'b> From<DecodedData<'b>> for ByteList {
  fn from(value: DecodedData<'b>) -> Self {
    if value.entity_count() == 0 {
      return value.inner_bytes.into_owned();
    }
    (&value).into()
  }
}

/**
 * impl decoded data into char list
 */
impl<'b> From<&DecodedData<'b>> for CharListResult {
  fn from(value: &DecodedData<'b>) -> Self {
    call_into_char_list_trait(&value.inner_bytes, &value.entities, |&(ch, _), result| {
      result.push(ch)
    })
  }
}

impl<'b> From<DecodedData<'b>> for CharListResult {
  fn from(value: DecodedData<'b>) -> Self {
    (&value).into()
  }
}
/// EncodedData, impl the ICodedDataTrait and IBytesTrait and IntoIterator.
#[derive(Debug)]
pub struct EncodedData<'b> {
  inner_bytes: Cow<'b, [Byte]>,
  entities: Vec<(CodeRange, CharEntity)>,
}

impl<'b> ICodedDataTrait for EncodedData<'b> {}

impl<'b> IBytesTrait for EncodedData<'b> {
  fn byte(&self, index: usize) -> Option<&Byte> {
    call_trait_method_byte(&self.inner_bytes, &self.entities, index)
  }
  fn bytes_len(&self) -> usize {
    call_trait_method_bytes_len(&self.inner_bytes, &self.entities)
  }
}

impl<'b> EncodedData<'b> {
  // detect
  pub fn entity_count(&self) -> usize {
    self.entities.len()
  }
  // to owned
  pub fn to_owned(&mut self) {
    if self.entities.is_empty() {
      self.inner_bytes = self.inner_bytes.clone();
    } else {
      let bytes = self.to_bytes();
      self.inner_bytes = Cow::Owned(bytes);
      self.entities.clear();
    }
  }
  // into bytes
  pub fn into_bytes(self) -> ByteList {
    if self.entities.is_empty() {
      return self.inner_bytes.into_owned();
    }
    self.to_bytes()
  }
  // get bytes with cow
  pub fn bytes(&self) -> Cow<'b, [Byte]> {
    if self.entities.is_empty() {
      return self.inner_bytes.clone();
    }
    return Cow::Owned(self.to_bytes());
  }
}

impl<'b> From<&EncodedData<'b>> for StringResult {
  fn from(value: &EncodedData<'b>) -> Self {
    call_into_string_trait(
      &value.inner_bytes,
      &value.entities,
      |char_entity, result| {
        char_entity.write_string(result);
      },
    )
  }
}

impl<'b> From<EncodedData<'b>> for StringResult {
  fn from(value: EncodedData<'b>) -> Self {
    (&value).into()
  }
}

impl<'b> From<&EncodedData<'b>> for CharListResult {
  fn from(value: &EncodedData<'b>) -> Self {
    call_into_char_list_trait(
      &value.inner_bytes,
      &value.entities,
      |char_entity, result| {
        char_entity.write_chars(result);
      },
    )
  }
}

impl<'b> From<EncodedData<'b>> for CharListResult {
  fn from(value: EncodedData<'b>) -> Self {
    (&value).into()
  }
}

impl<'b> From<&EncodedData<'b>> for ByteList {
  fn from(value: &EncodedData<'b>) -> Self {
    value
      .into_iter()
      .map(|(byte, _)| *byte)
      .collect::<ByteList>()
  }
}

impl<'b> From<EncodedData<'b>> for ByteList {
  fn from(value: EncodedData<'b>) -> Self {
    if value.entity_count() == 0 {
      return value.inner_bytes.into_owned();
    }
    (&value).into()
  }
}

/// EncodeType: html entity encoding format
#[derive(Copy, Clone, Default)]
#[repr(u8)]
pub enum EncodeType {
  #[default]
  Named = 0b00001,
  Hex = 0b00010,
  Decimal = 0b00100,
  NamedOrHex = 0b00011,
  NamedOrDecimal = 0b00101,
}

#[inline]
fn filter_entity_set(
  charset: &EntityCharBytes,
  encode_type: &EncodeType,
  ch: &char,
) -> EncodeFilterReturnData {
  let encode_type = *encode_type as u8;
  if let Some(&v) = charset.get(ch) {
    if (encode_type & EncodeType::Named as u8) > 0 {
      return (true, Some((EntityType::Named, Cow::from(v))));
    }
    return (true, None);
  }
  (false, None)
}

/// The character set that needs to be encoded to html entity.
#[derive(Default)]
pub enum CharacterSet {
  /// all characters
  All = 1,
  /// non ASCII, code point > 0xff                
  NonASCII = 2,
  /// html: '<','>','&'    
  #[default]
  Html = 3,
  /// special characters: '<','>','&', '\'', '"'                
  SpecialChars = 4,
  /// html and non ascii
  HtmlAndNonASCII = 5,
  /// special characters and non ascii
  SpecialCharsAndNonASCII = 6,
}

impl CharacterSet {
  /// check if a character need encode by the encode type, and encode it if nessessary.
  pub fn filter(&self, ch: &char, encode_type: &EncodeType) -> EncodeFilterReturnData {
    use CharacterSet::*;
    match self {
      SpecialChars => filter_entity_set(&SPECIAL_BYTES, encode_type, ch),
      Html => filter_entity_set(&HTML_BYTES, encode_type, ch),
      NonASCII => (*ch as u32 > 0xff, None),
      HtmlAndNonASCII => {
        let result = CharacterSet::NonASCII.filter(ch, encode_type);
        if result.0 {
          return result;
        }
        CharacterSet::Html.filter(ch, encode_type)
      }
      SpecialCharsAndNonASCII => {
        let result = CharacterSet::NonASCII.filter(ch, encode_type);
        if result.0 {
          return result;
        }
        CharacterSet::SpecialChars.filter(ch, encode_type)
      }
      All => (true, None),
    }
  }
  /// Check if the character is in the charcter set
  pub fn contains(&self, ch: &char) -> bool {
    use CharacterSet::*;
    match self {
      SpecialChars => SPECIAL_BYTES.get(ch).is_some(),
      Html => HTML_BYTES.get(ch).is_some(),
      NonASCII => *ch as u32 > 0xff,
      HtmlAndNonASCII => CharacterSet::NonASCII.contains(ch) || CharacterSet::Html.contains(ch),
      SpecialCharsAndNonASCII => {
        CharacterSet::NonASCII.contains(ch) || CharacterSet::SpecialChars.contains(ch)
      }
      All => true,
    }
  }
}

#[derive(PartialEq, Eq, Debug)]
pub enum EntityType {
  Named,
  Hex,
  Decimal,
}

/// CharEntity struct
#[derive(Debug)]
pub struct CharEntity {
  entity_type: EntityType,
  entity_data: Cow<'static, [Byte]>,
}

impl CharEntity {
  // prefix len
  pub fn prefix_len(&self) -> usize {
    match &self.entity_type {
      EntityType::Named => 0,
      EntityType::Hex => 2,
      EntityType::Decimal => 1,
    }
  }
  // write bytes
  pub fn write_bytes(&self, bytes: &mut ByteList) {
    bytes.push(b'&');
    match &self.entity_type {
      EntityType::Named => {
        // nothing to do
      }
      EntityType::Hex => {
        bytes.push(b'#');
        bytes.push(b'x');
      }
      EntityType::Decimal => {
        bytes.push(b'#');
      }
    }
    bytes.extend_from_slice(&self.entity_data);
    bytes.push(b';');
  }
  // write chars
  pub fn write_chars(&self, chars: &mut Vec<char>) {
    chars.push('&');
    match &self.entity_type {
      EntityType::Named => {
        // nothing to do
      }
      EntityType::Hex => {
        chars.push('#');
        chars.push('x');
      }
      EntityType::Decimal => {
        chars.push('#');
      }
    }
    for byte in self.entity_data.iter() {
      chars.push(*byte as char);
    }
    chars.push(';');
  }
  // write string
  pub fn write_string(&self, code: &mut String) {
    code.push('&');
    match &self.entity_type {
      EntityType::Named => {
        // nothing to do
      }
      EntityType::Hex => {
        code.push('#');
        code.push('x');
      }
      EntityType::Decimal => {
        code.push('#');
      }
    }
    for byte in self.entity_data.iter() {
      code.push(*byte as char);
    }
    code.push(';');
  }
  // to bytes
  pub fn to_bytes(&self) -> ByteList {
    let mut bytes: ByteList = Vec::with_capacity(self.entity_data.len() + 2);
    self.write_bytes(&mut bytes);
    bytes
  }
  // get out of entity_data
  pub fn data(self) -> ByteList {
    self.entity_data.into_owned()
  }
}

impl ToString for CharEntity {
  fn to_string(&self) -> String {
    let mut code = String::with_capacity(self.entity_data.len() + 2);
    self.write_string(&mut code);
    code
  }
}
/// Entity struct
#[derive(Default)]
pub struct Entity;

impl Entity {
  /// Decode html entity utf-8 bytes(does't contain the beginning '&' and the end ';') into the character.
  pub fn decode(bytes: &[Byte]) -> AnyhowResult<char> {
    let total = bytes.len();
    if total == 0 {
      return Err(
        HtmlEntityError::Decode(String::from(
          "Can't decode with an empty bytelist argument.",
        ))
        .into(),
      );
    }
    // check type
    let first = bytes[0];
    let mut entity_type: EntityType = EntityType::Named;
    if first.is_ascii_alphabetic() {
      for ch in &bytes[1..] {
        if !ch.is_ascii_alphanumeric() {
          let code = std::str::from_utf8(bytes)?;
          return Err(
            HtmlEntityError::Decode(format!(
							"Html entity name can't contain characters other than English letters or numbers, here is '{}'",
							code
						))
            .into(),
          );
        }
      }
    } else if first == b'#' && total > 1 {
      let second = bytes[1];
      match second {
        b'0'..=b'9' => {
          // decimal
          for byte in &bytes[2..] {
            if !byte.is_ascii_digit() {
              let code = std::str::from_utf8(bytes)?;
              return Err(
                HtmlEntityError::Decode(format!(
                  "Html entity number can't contain characters other than numbers, here is '{}'.",
                  code
                ))
                .into(),
              );
            }
          }
          entity_type = EntityType::Decimal;
        }
        b'x' | b'X' => {
          // hex
          if total > 2 {
            for byte in &bytes[2..] {
              if !matches!(byte, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9') {
                let code = std::str::from_utf8(bytes)?;
                return Err(
                  HtmlEntityError::Decode(format!(
										"Hexadecimal html entity can't contain characters other than hexadecimal, here is '&{};'.",
										code
									))
                  .into(),
                );
              }
            }
            entity_type = EntityType::Hex;
          } else {
            return Err(
              HtmlEntityError::Decode(String::from(
                "Hexadecimal html entity must contain one or more hexadecimal characters.",
              ))
              .into(),
            );
          }
        }
        _ => {
          return Err(
            HtmlEntityError::Decode(String::from("Illegal html entity number character format"))
              .into(),
          );
        }
      }
    } else {
      return Err(
        HtmlEntityError::Decode(String::from("Illegal html entity character format.")).into(),
      );
    }
    // go on the steps
    match entity_type {
      // named entity
      EntityType::Named => {
        // normal entity characters
        if let Some(&ch) = NORMAL_NAME_ENTITY_BYTE.get(bytes) {
          return Ok(ch);
        }
        // try to find the entity
        if let Some(&(start_index, end_index)) = FIRST_LETTER_POSITION.get(&bytes[0]) {
          if let Some(find_index) = LETTER_ORDERED_ENTITIES[start_index..end_index]
            .iter()
            .position(|&(name, _)| name == bytes)
          {
            let last_index = start_index + find_index;
            let (_, code) = LETTER_ORDERED_ENTITIES[last_index];
            return Ok(code);
          }
        }
        let code = std::str::from_utf8(bytes)?;
        Err(
          HtmlEntityError::Decode(format!(
            "Unable to find corresponding the html entity name '&{};'",
            code
          ))
          .into(),
        )
      }
      // hex entity
      EntityType::Hex => {
        // remove the prefix '#x'
        numbers_to_char(&bytes[2..], 16)
      }
      // decimal entity
      EntityType::Decimal => {
        // remove the prefix '#'
        numbers_to_char(&bytes[1..], 10)
      }
    }
  }
  /// Similar to the `decode` method, but takes a character type as an argument.
  pub fn decode_chars(chars: &[char]) -> AnyhowResult<char> {
    let total = chars.len();
    if total == 0 {
      return Err(
        HtmlEntityError::Decode(String::from(
          "Can't decode with an empty character list argument.",
        ))
        .into(),
      );
    }
    let mut bytes: ByteList = Vec::with_capacity(total);
    let max_u8 = u8::MAX as u32;
    let is_non_bytes = chars.iter().any(|ch| {
      let char_code = *ch as u32;
      if char_code > max_u8 {
        true
      } else {
        bytes.push(char_code as Byte);
        false
      }
    });
    if !is_non_bytes {
      return Entity::decode(&bytes);
    }
    Err(
      HtmlEntityError::Decode(format!(
        "Unable to find corresponding the html entity name '&{};'",
        chars.iter().collect::<String>()
      ))
      .into(),
    )
  }
}

/// Encode character into html entity.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let character = '<';
/// let char_entity = encode_char(&character, &EncodeType::Named);
/// assert!(char_entity.is_some());
/// assert_eq!(char_entity.unwrap().to_string(), "&lt;");
///
/// let character = '<';
/// let char_entity = encode_char(&character, &EncodeType::Decimal);
/// assert!(char_entity.is_some());
/// assert_eq!(char_entity.unwrap().to_string(), "&#60;");
///
/// let character = '<';
/// let char_entity = encode_char(&character, &EncodeType::Hex);
/// assert!(char_entity.is_some());
/// assert_eq!(char_entity.unwrap().to_string(), "&#x3c;");
/// ```
pub fn encode_char(ch: &char, encode_type: &EncodeType) -> Option<CharEntity> {
  let encode_type = *encode_type as u8;
  let char_code = *ch as u32;
  // encode to named
  if (encode_type & (EncodeType::Named as u8)) > 0 {
    // find the named entity from the ENTITIES
    if let Ok(mut index) = ENTITIES.binary_search_by_key(&char_code, |&(_, code)| code) {
      // make sure the entity is the first one, short and lowercase
      while index > 0 {
        let prev_index = index - 1;
        if ENTITIES[prev_index].1 != char_code {
          break;
        }
        index = prev_index;
      }
      let &(entity, _) = &ENTITIES[index];
      return Some(CharEntity {
        entity_type: EntityType::Named,
        entity_data: Cow::from(entity),
      });
    }
  }
  // encode to hex
  if (encode_type & (EncodeType::Hex as u8)) > 0 {
    return Some(CharEntity {
      entity_type: EntityType::Hex,
      entity_data: Cow::Owned(format!("{:x}", char_code).into_bytes()),
    });
  }
  // encode to decimal
  if (encode_type & (EncodeType::Decimal as u8)) > 0 {
    return Some(CharEntity {
      entity_type: EntityType::Decimal,
      entity_data: Cow::Owned(char_code.to_string().into_bytes()),
    });
  }
  // no need to encode or failure
  None
}

/// Encode characters in the utf-8 bytes into html entities according to the specified encoding format and specified encoding character set.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::AnyhowResult;
/// # fn main() -> AnyhowResult<()> {
/// let html = "<div class='header'></div>";
/// let encoded_data = encode(html.as_bytes(), &EncodeType::Named, &CharacterSet::SpecialChars);
/// // Convert encoded data to string.
/// let data_to_string = encoded_data.to_string();
/// assert!(data_to_string.is_ok());
/// assert_eq!(data_to_string?, "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
/// // Convert encoded data to Vec<char>.
/// let data_to_chars = encoded_data.to_chars();
/// assert!(data_to_chars.is_ok());
/// assert_eq!(data_to_chars?, String::from("&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;").chars().collect::<Vec<char>>());
/// // Convert encoded data to bytes(Vec<u8>).
/// let data_to_bytes = encoded_data.to_bytes();
/// let bytes = b"&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;";
/// assert_eq!(data_to_bytes, bytes);
/// // Encoded data can be iterated by byte
/// for (idx, (byte, _)) in encoded_data.into_iter().enumerate(){
///    assert_eq!(*byte, bytes[idx]);
/// }
/// // Get the total bytes size through the 'bytes_len' method and visit the byte through the 'byte(idx)' method.
/// for idx in 0..encoded_data.bytes_len(){
///    assert_eq!(encoded_data.byte(idx), Some(&bytes[idx]));
/// }
/// # Ok(())
/// # }
/// ```
pub fn encode<'a>(
  content: &'a [Byte],
  encode_type: &EncodeType,
  charset: &CharacterSet,
) -> EncodedData<'a> {
  encode_with(content, encode_type, |ch, encode_type| {
    charset.filter(ch, encode_type)
  })
}

/// Similar to the `encode` method, but directly writes the byte data into the last parameter passed in.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::{ ByteList, AnyhowResult };
///
/// let html = "<div class='header'></div>";
/// let mut data: ByteList = vec![];
/// encode_to(html.as_bytes(), &EncodeType::Named, &CharacterSet::SpecialChars, &mut data);
/// assert_eq!(data, b"&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
/// ```
pub fn encode_to(
  content: &[Byte],
  encode_type: &EncodeType,
  charset: &CharacterSet,
  data: &mut ByteList,
) {
  encode_with_to(
    content,
    encode_type,
    |ch, encode_type| charset.filter(ch, encode_type),
    data,
  );
}

/// Encode the html entities in utf-8 bytes into encoded data, and specify the characters to be encoded and the encoding format through the `filter_fn` method parameter.
///
/// # Examples
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::AnyhowResult;
/// use std::borrow::Cow;
/// # fn main() -> AnyhowResult<()> {
/// let html = "<div class='header'></div>";
/// let charset = CharacterSet::SpecialChars;
/// let encoded_data = encode_with(&html.as_bytes(), &EncodeType::Named, |ch, encode_type|{
///    // Use html entity number encoding for single quotes (')
///    if ch == &'\''{
///       if let Some(char_entity) = encode_char(ch, &EncodeType::Decimal){
///         return (true, Some((EntityType::Decimal, Cow::from(char_entity.data()))));
///       }
///    }
///    return charset.filter(ch, encode_type);
/// });
/// let data_to_string = encoded_data.to_string();
/// assert!(data_to_string.is_ok());
/// assert_eq!(data_to_string?, String::from("&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;"));
/// # Ok(())
/// # }
/// ```
pub fn encode_with<'a>(
  content: &'a [Byte],
  encode_type: &EncodeType,
  filter_fn: impl Fn(&char, &EncodeType) -> EncodeFilterReturnData,
) -> EncodedData<'a> {
  let mut entities: Vec<(CodeRange, CharEntity)> = vec![];
  let _ = loop_utf8_bytes(content, |result, (start_index, index)| match result {
    Utf8ParsedData::Correct(ch) => {
      let (need_encode, maybe_entity) = filter_fn(&ch, encode_type);
      if need_encode {
        if let Some((entity_type, entity_data)) = maybe_entity {
          entities.push((
            start_index..=index,
            CharEntity {
              entity_type,
              entity_data,
            },
          ));
        } else if let Some(entity) = encode_char(&ch, encode_type) {
          entities.push((start_index..=index, entity));
        }
      }
      Ok(())
    }
    _ => Ok(()),
  });
  EncodedData {
    inner_bytes: Cow::from(content),
    entities,
  }
}

/// Similar to the `encode_with` method, but directly writes the byte data into the last parameter passed in.
///
/// # Examples
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::{ ByteList, AnyhowResult };
/// use std::borrow::Cow;
/// # fn main() -> AnyhowResult<()> {
/// let html = "<div class='header'></div>";
/// let charset = CharacterSet::SpecialChars;
/// let mut data: ByteList = vec![];
/// let encoded_data = encode_with_to(&html.as_bytes(), &EncodeType::Named, |ch, encode_type|{
///    return charset.filter(ch, encode_type);
/// }, &mut data);
/// assert_eq!(data, b"&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
/// # Ok(())
/// # }
/// ```
pub fn encode_with_to(
  content: &[Byte],
  encode_type: &EncodeType,
  filter_fn: impl Fn(&char, &EncodeType) -> EncodeFilterReturnData,
  data: &mut ByteList,
) {
  let _ = loop_utf8_bytes(content, |result, (start_index, end_index)| match result {
    Utf8ParsedData::Correct(ch) => {
      let (need_encode, maybe_entity) = filter_fn(&ch, encode_type);
      if need_encode {
        if let Some((entity_type, entity_data)) = maybe_entity {
          let entity = CharEntity {
            entity_type,
            entity_data,
          };
          entity.write_bytes(data);
          return Ok(());
        } else if let Some(entity) = encode_char(&ch, encode_type) {
          entity.write_bytes(data);
          return Ok(());
        }
      }
      data.extend_from_slice(&content[start_index..=end_index]);
      Ok(())
    }
    Utf8ParsedData::Wrong(_) => {
      data.extend_from_slice(&content[start_index..=end_index]);
      Ok(())
    }
  });
}

/// Encode a list of characters using a filter function.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use std::borrow::Cow;
///
/// let chars = String::from("<div class='header'></div>").chars().collect::<Vec<char>>();
/// let character_set = CharacterSet::HtmlAndNonASCII;
/// let encoded_chars = encode_chars_with(&chars, |ch|{
///   if character_set.contains(ch) || *ch == '\''{
///      return Some(&EncodeType::Named);
///   }
///   return None;
/// });
/// assert_eq!(encoded_chars.iter().collect::<String>(), "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
/// ```
pub fn encode_chars_with(
  chars: &[char],
  filter_fn: impl Fn(&char) -> Option<&EncodeType>,
) -> Cow<'_, [char]> {
  let mut result = vec![];
  let mut iter = chars.iter();
  for (index, ch) in iter.by_ref().enumerate() {
    if let Some(encode_type) = filter_fn(ch) {
      if let Some(entity) = encode_char(ch, encode_type) {
        if index > 0 {
          result.extend_from_slice(&chars[..index]);
        }
        entity.write_chars(&mut result);
        break;
      }
    }
  }
  for ch in iter {
    if let Some(encode_type) = filter_fn(ch) {
      if let Some(entity) = encode_char(ch, encode_type) {
        entity.write_chars(&mut result);
        continue;
      }
    }
    result.push(*ch);
  }
  if !result.is_empty() {
    return Cow::Owned(result);
  }
  Cow::Borrowed(chars)
}

/// Decode the html entities in the character list.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use std::borrow::Cow;
///
/// let char_list = Cow::from(vec!['a', '<', 'b']);
/// assert_eq!(decode_chars(&String::from("a&lt;b").chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars(&String::from("a&#60;b").chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars(&String::from("a&#x3c;b").chars().collect::<Vec<char>>()), char_list);
/// ```
pub fn decode_chars(chars: &[char]) -> Cow<'_, [char]> {
  let mut data: Vec<char> = vec![];
  let mut is_in_entity = false;
  let mut start_index: usize = 0;
  for (idx, ch) in chars.iter().enumerate() {
    if !is_in_entity {
      // not in entity
      if *ch == '&' {
        is_in_entity = true;
        start_index = idx + 1;
      }
    } else {
      // in entity
      match *ch {
        ';' => {
          // end of the entity, ignore '&;'
          if start_index != idx {
            let bytes = tr_chars_to_utf8_bytes(&chars[start_index..idx]);
            if let Some(bytes) = bytes {
              if let Ok(decode_char) = Entity::decode(&bytes) {
                // find at least one entity
                // append the entity's prev chars
                if start_index > 1 {
                  data.extend_from_slice(&chars[..start_index - 1]);
                }
                // append entity character
                data.push(decode_char);
                // append the left character
                let next_idx = idx + 1;
                if next_idx != chars.len() {
                  decode_chars_to(&chars[next_idx..], &mut data);
                }
                return Cow::Owned(data);
              }
            }
          }
          is_in_entity = false;
        }
        '&' => {
          // always reset entity start index
          start_index = idx + 1;
        }
        _ => {}
      }
    }
  }
  Cow::from(chars)
}

/// Similar to the `decode_chars` method, but directly writes the character data into the last parameter passed in.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use std::borrow::Cow;
///
/// let char_list = vec!['a','<', 'b'];
/// let mut data: Vec<char> = vec![];
/// decode_chars_to(&String::from("a&lt;b").chars().collect::<Vec<char>>(), &mut data);
/// assert_eq!(data, char_list);
///
/// data.clear();
/// decode_chars_to(&String::from("a&#60;b").chars().collect::<Vec<char>>(), &mut data);
/// assert_eq!(data, char_list);
///
/// data.clear();
/// decode_chars_to(&String::from("a&#x3c;b").chars().collect::<Vec<char>>(), &mut data);
/// assert_eq!(data, char_list);
/// ```
pub fn decode_chars_to(chars: &[char], data: &mut Vec<char>) {
  let mut is_in_entity = false;
  let mut start_index: usize = 0;
  for (idx, &ch) in chars.iter().enumerate() {
    if !is_in_entity {
      // not in entity
      if ch == '&' {
        is_in_entity = true;
        start_index = idx + 1;
      } else {
        data.push(ch);
      }
    } else {
      // in entity
      match ch {
        ';' => {
          // end of the entity, ignore '&;'
          if start_index != idx {
            let bytes = tr_chars_to_utf8_bytes(&chars[start_index..idx]);
            if let Some(bytes) = bytes {
              if let Ok(decode_char) = Entity::decode(&bytes) {
                // find the
                data.push(decode_char);
                is_in_entity = false;
                continue;
              }
            }
          }
          // not a regular entity
          data.extend_from_slice(&chars[start_index - 1..=idx]);
          is_in_entity = false;
        }
        '&' => {
          // always reset the entity start index, '&a&lt;'
          data.extend_from_slice(&chars[start_index - 1..idx]);
          start_index = idx + 1;
        }
        _ => {}
      }
    }
  }
  if is_in_entity {
    // add the end non regular entity
    data.extend_from_slice(&chars[start_index - 1..]);
  }
}

/// Decode html entities in utf-8 bytes.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::AnyhowResult;
/// # fn main() -> AnyhowResult<()> {
/// let html = "<div class='header'></div>";
/// let orig_bytes = html.as_bytes();
/// let encoded_data = encode(orig_bytes, &EncodeType::Named, &CharacterSet::SpecialChars);
/// let encode_bytes = encoded_data.to_bytes();
/// assert_eq!(encode_bytes, b"&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
/// // decode the bytes
/// let decoded_data = decode(&encode_bytes);
/// let data_to_string = decoded_data.to_string();
/// assert!(data_to_string.is_ok());
/// assert_eq!(data_to_string?, String::from(html));
/// // Convert encoded data to Vec<char>.
/// let data_to_chars = decoded_data.to_chars();
/// assert!(data_to_chars.is_ok());
/// assert_eq!(data_to_chars?, String::from(html).chars().collect::<Vec<char>>());
/// // Convert encoded data to bytes(Vec<u8>).
/// let data_to_bytes = decoded_data.to_bytes();
/// assert_eq!(data_to_bytes, html.as_bytes());
/// // Decoded data can be also iterated by byte
/// for (idx, (byte, _)) in decoded_data.into_iter().enumerate(){
///    assert_eq!(*byte, orig_bytes[idx]);
/// }
/// // Get the total bytes size through the 'bytes_len' method and visit the byte through the 'byte(idx)' method.
/// for idx in 0..decoded_data.bytes_len(){
///    assert_eq!(decoded_data.byte(idx), Some(&orig_bytes[idx]));
/// }
/// # Ok(())
/// # }
/// ```
/// ```
pub fn decode(content: &[Byte]) -> DecodedData<'_> {
  let mut entities: Vec<(CodeRange, (char, ByteList))> = vec![];
  let mut errors: Vec<(CodeRange, anyhow::Error)> = vec![];
  let mut is_in_entity = false;
  let mut start_index: usize = 0;
  for (idx, byte) in content.iter().enumerate() {
    if !is_in_entity {
      // not in entity
      if *byte == b'&' {
        is_in_entity = true;
        start_index = idx + 1;
      }
    } else {
      // in entity
      match *byte {
        b';' => {
          // end of the entity, ignore '&;'
          if start_index != idx {
            let decode_result = Entity::decode(&content[start_index..idx]);
            match decode_result {
              Ok(decode_char) => {
                entities.push((
                  start_index - 1..=idx,
                  (decode_char, char_to_utf8_bytes(decode_char)),
                ));
              }
              Err(err) => {
                errors.push((start_index - 1..=idx, err));
              }
            };
          }
          is_in_entity = false;
        }
        b'&' => {
          // always reset entity start index
          errors.push((
            start_index - 1..=start_index - 1,
            HtmlEntityError::Decode(String::from("Unencoded html entity characters '&'.")).into(),
          ));
          start_index = idx + 1;
        }
        _ => {
          // entity bytes
        }
      }
    }
  }
  // wrong entity at the end
  DecodedData {
    inner_bytes: Cow::from(content),
    entities,
    errors,
  }
}

/// Similar to the `decode` method, but directly writes the byte data into the last parameter passed in.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
/// use htmlentity::types::ByteList;
/// use std::borrow::Cow;
///
/// let encoded_bytes = b"&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;";
/// let mut data: ByteList = vec![];
/// decode_to(encoded_bytes, &mut data);
/// assert_eq!(data, b"<div class='header'></div>");
/// ```
pub fn decode_to(content: &[Byte], data: &mut Vec<Byte>) {
  let mut is_in_entity = false;
  let mut start_index: usize = 0;
  for (idx, byte) in content.iter().enumerate() {
    if !is_in_entity {
      // not in entity
      if *byte == b'&' {
        is_in_entity = true;
        start_index = idx + 1;
      } else {
        data.push(*byte);
      }
    } else {
      // in entity
      match *byte {
        b';' => {
          // end of the entity, ignore '&;'
          if start_index != idx {
            if let Ok(decode_char) = Entity::decode(&content[start_index..idx]) {
              data.extend(char_to_utf8_bytes(decode_char));
              is_in_entity = false;
              continue;
            }
          }
          data.extend_from_slice(&content[start_index - 1..=idx]);
          is_in_entity = false;
        }
        b'&' => {
          // always reset entity start index
          data.extend_from_slice(&content[start_index - 1..idx]);
          start_index = idx + 1;
        }
        _ => {
          // entity bytes
        }
      }
    }
  }
  if is_in_entity {
    // add the end non regular entity
    data.extend_from_slice(&content[start_index - 1..]);
  }
}
