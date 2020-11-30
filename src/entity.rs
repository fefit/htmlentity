use crate::data::{EntityItem, ENTITIES};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

type SortedEntity = Vec<EntityItem>;
type Positions = HashMap<u8, (usize, usize)>;

/// NOOP is the None value of Option<dyn Fn(char)->bool>  
pub const NOOP: Option<&dyn Fn(char) -> bool> = None::<&dyn Fn(char) -> bool>;

lazy_static! {
  static ref IS_SORTED: AtomicBool = AtomicBool::new(false);
  static ref DECODE_ENTITIES: Mutex<SortedEntity> = Mutex::new(vec![]);
  static ref FIRST_POSITION: Mutex<Positions> = Mutex::new(HashMap::new());
  // special chars
  static ref SPECIAL_CHARS: HashMap<char, &'static str> = {
    let mut map = HashMap::new();
    map.insert('>', "&gt;");
    map.insert('<', "&lt;");
    map.insert('"', "&quot;");
    map.insert('\'', "&apos;");
    map.insert('&', "&amp;");
    map
  };
}

/// Encode a character.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let character = '<';
/// let char_encoded = encode_char(character, EncodeType::Named, NOOP);
/// assert_eq!(char_encoded, "&lt;");
///
/// let character = '<';
/// let char_encoded = encode_char(character, EncodeType::Decimal, NOOP);
/// assert_eq!(char_encoded, "&#60;");
///
/// let character = '<';
/// let char_encoded = encode_char(character, EncodeType::Hex, NOOP);
/// assert_eq!(char_encoded, "&#x3c;");
///
/// let character = '<';
/// let char_encoded = encode_char(character, EncodeType::Named, Some(|ch|ch == '<'));
/// assert_eq!(char_encoded, "<");
/// ```
pub fn encode_char<F>(ch: char, encode_type: EncodeType, exclude_fn: Option<F>) -> String
where
  F: Fn(char) -> bool,
{
  use EncodeType::*;
  let encode_type = encode_type as u8;
  let char_code = ch as u32;
  let mut result = String::with_capacity(5);
  if encode_type & (Named as u8) > 0 {
    let mut should_find_name = true;
    if let Some(exclude_fn) = exclude_fn {
      if exclude_fn(ch) {
        should_find_name = false;
      }
    }
    if should_find_name {
      let finded = (&ENTITIES[..]).binary_search_by_key(&char_code, |&(_, code)| code);
      if let Ok(index) = finded {
        let mut first_index = index;
        // find the first, short and lowercase
        loop {
          if first_index > 0 {
            let next_index = first_index - 1;
            let (_, cur_char_code) = ENTITIES[next_index];
            if cur_char_code != char_code {
              break;
            }
            first_index -= 1;
          } else {
            break;
          }
        }
        let (entity, _) = ENTITIES[first_index];
        result.push('&');
        result.push_str(entity);
        result.push(';');
        return result;
      }
    }
  }
  if encode_type & (Hex as u8) > 0 {
    let hex = format!("&#x{:x};", char_code);
    result.push_str(&hex);
    return result;
  }
  if encode_type & (Decimal as u8) > 0 {
    let dec = format!("&#{};", char_code);
    result.push_str(&dec);
    return result;
  }
  result.push(ch);
  result
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
/// The type of characters you need encoded, default: `SpecialCharsAndNoASCII`
pub enum Entities {
  SpecialCharsAndNoASCII = 6, // default
  All = 1,                    // encode all
  NoASCII = 2,                // encode character not ascii
  SpecialChars = 4,           // encode '<>&''
}

impl Default for Entities {
  fn default() -> Self {
    Entities::SpecialCharsAndNoASCII
  }
}

impl Entities {
  pub fn filter(&self, ch: &char, encode_type: EncodeType) -> (bool, Option<String>) {
    use Entities::*;
    match self {
      SpecialChars => {
        let encode_type = encode_type as u8;
        if let Some(&v) = SPECIAL_CHARS.get(ch) {
          if (encode_type & EncodeType::Named as u8) > 0 {
            return (true, Some(v.into()));
          }
          return (true, None);
        }
        (false, None)
      }
      NoASCII => (*ch as u32 > 0x80, None),
      SpecialCharsAndNoASCII => {
        let (need_encode, result) = Entities::NoASCII.filter(ch, encode_type);
        if need_encode {
          return (need_encode, result);
        }
        Entities::SpecialChars.filter(ch, encode_type)
      }
      All => (true, None),
    }
  }
  pub fn contains(&self, ch: &char) -> bool {
    let (flag, _) = self.filter(ch, EncodeType::Decimal);
    flag
  }
}

/// Encode a html code's characters into entities.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let html = "<div class='header'></div>";
/// let html_encoded = encode(html, Entities::SpecialChars, EncodeType::Named);
/// assert_eq!(html_encoded, "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
///
/// let html_decoded = decode(&html_encoded);
/// assert_eq!(html, html_decoded);
/// ```
pub fn encode(content: &str, entities: Entities, encode_type: EncodeType) -> String {
  let mut result = String::with_capacity(content.len() + 5);
  for ch in content.chars() {
    let (need_encode, encoded) = entities.filter(&ch, encode_type);
    if need_encode {
      if let Some(encoded) = encoded {
        result.push_str(&encoded);
      } else {
        let encoded = encode_char(ch, encode_type, NOOP);
        result.push_str(&encoded);
      }
    } else {
      result.push(ch);
    }
  }
  result
}

/// Short for `encode(content, Entities::default(), EncodeType::default())`
pub fn encode_default(content: &str) -> String {
  encode(content, Default::default(), Default::default())
}

/// Encode by filter functions.
/// Use the `filte_fn` to choose the character need to encode.
/// Use the `exclude_fn` to exclude characters you don't want to use named.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let html = "<div class='header'></div>";
/// let html_encoded = encode_filter(html, |ch|{
///   // special characters but not '<'
///   ch != '<' && Entities::SpecialChars.contains(&ch)
/// }, EncodeType::Named, NOOP);
/// assert_eq!(html_encoded, "<div class=&apos;header&apos;&gt;</div&gt;");
///
/// // special characters, but exclude the single quote "'" use named.
/// let html = "<div class='header'></div>";
/// let html_encoded = encode_filter(html, |ch|{
///   Entities::SpecialChars.contains(&ch)
/// }, EncodeType::NamedOrDecimal, Some(|ch| ch == '\''));
/// assert_eq!(html_encoded, "&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;");
/// ```
pub fn encode_filter<F: Fn(char) -> bool, C: Fn(char) -> bool>(
  content: &str,
  filter_fn: F,
  encode_type: EncodeType,
  exclude_fn: Option<C>,
) -> String {
  let mut result = String::with_capacity(content.len() + 5);
  for ch in content.chars() {
    if filter_fn(ch) {
      result.push_str(&encode_char(ch, encode_type, exclude_fn.as_ref()));
    } else {
      result.push(ch);
    }
  }
  result
}

/**
 * Sort
 * 将entities排序成所需格式
 */
fn sort_entities() {
  let mut sorted: SortedEntity = Vec::with_capacity(ENTITIES.len());
  let mut counts: Positions = HashMap::new();
  let mut firsts: Vec<u8> = Vec::with_capacity(52);
  // 二分查找插入
  for pair in &ENTITIES[..] {
    let entity = *pair;
    let chars = entity.0.as_bytes();
    let first = chars[0];
    binary_insert(&mut sorted, entity);
    // 插入首字母个数到hashmap
    match counts.get_mut(&first) {
      Some((v, _)) => {
        *v += 1;
      }
      None => {
        counts.insert(first, (1, 0));
      }
    }
    // 插入首字母到表
    if !firsts.contains(&first) {
      firsts.push(first);
    }
  }
  // 整理首位序号
  firsts.sort_unstable();
  let mut cur_index: usize = 0;
  for char_code in firsts {
    let position = counts.get_mut(&char_code).unwrap();
    let next_index = cur_index + position.0;
    *position = (cur_index, next_index);
    cur_index = next_index;
  }
  // 赋值位置的HashMap
  let mut positions = FIRST_POSITION.lock().unwrap();
  *positions = counts;
  // 赋值排序好的实体
  let mut entities = DECODE_ENTITIES.lock().unwrap();
  *entities = sorted;
}
/**
 * 二分查找插入
 */
fn binary_insert(sorted: &mut SortedEntity, cur: EntityItem) {
  let mut prev_index = 0;
  let len = sorted.len();
  if len > 0 {
    let search = cur.0;
    prev_index = match sorted[..].binary_search_by(|&(name, _)| name.cmp(search)) {
      Ok(index) => index,
      Err(index) => index,
    };
  }
  (*sorted).insert(prev_index, cur);
}

#[derive(PartialEq, Eq)]
pub enum EntityIn {
  Unkown,
  Named,
  Hex,
  Decimal,
  HexOrDecimal,
}
/// EncodeType: the output format type, default: `NamedOrDecimal`
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone)]
pub enum EncodeType {
  Named = 0b00001,
  Hex = 0b00010,
  Decimal = 0b00100,
  NamedOrHex = 0b00011,
  NamedOrDecimal = 0b00101,
}

impl Default for EncodeType {
  fn default() -> Self {
    EncodeType::NamedOrDecimal
  }
}

/// Decode character list, replace the entity characters into a unicode character.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let char_list = vec!['<'];
/// assert_eq!(decode_chars("&lt;".chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars("&#60;".chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars("&#x3c;".chars().collect::<Vec<char>>()), char_list);
/// ```
pub fn decode_chars(chars: Vec<char>) -> Vec<char> {
  use EntityIn::*;
  let is_sorted = IS_SORTED.load(Ordering::SeqCst);
  if !is_sorted {
    sort_entities();
    IS_SORTED.store(true, Ordering::SeqCst);
  }
  let sorted = DECODE_ENTITIES.lock().unwrap();
  let firsts = FIRST_POSITION.lock().unwrap();
  let mut result: Vec<char> = Vec::with_capacity(chars.len());
  let mut entity: Vec<char> = Vec::with_capacity(5);
  let mut is_in_entity: bool = false;
  let mut entity_in: EntityIn = Unkown;
  for ch in chars {
    if !is_in_entity {
      if ch == '&' {
        is_in_entity = true;
      } else {
        result.push(ch);
      }
    } else {
      let mut is_entity_complete = false;
      if ch != ';' {
        match entity_in {
          Named => {
            if !ch.is_ascii_alphabetic() {
              is_in_entity = false;
            }
          }
          Hex | Decimal => match ch {
            '0'..='9' => {}
            'a'..='f' | 'A'..='F' if entity_in == Hex => {}
            _ => {
              is_in_entity = false;
            }
          },
          Unkown => {
            if ch.is_ascii_alphabetic() {
              entity_in = Named;
            } else if ch == '#' {
              entity_in = HexOrDecimal;
            } else {
              is_in_entity = false;
            }
          }
          HexOrDecimal => match ch {
            '0'..='9' => {
              entity_in = Decimal;
            }
            'x' | 'X' => {
              entity_in = Hex;
            }
            _ => {
              is_in_entity = false;
            }
          },
        }
        if is_in_entity {
          entity.push(ch);
          continue;
        }
      } else {
        // end of the entity
        match entity_in {
          Named => {
            // try to find the entity
            let first = entity[0] as u32 as u8;
            if let Some(&(start_index, end_index)) = firsts.get(&first) {
              let searched = entity.iter().collect::<String>();
              if let Ok(find_index) = sorted[start_index..end_index]
                .binary_search_by(|&(name, _)| name.cmp(searched.as_str()))
              {
                let last_index = start_index + find_index;
                let (_, code) = sorted[last_index];
                result.push(std::char::from_u32(code).unwrap());
                is_entity_complete = true;
              }
            }
          }
          Hex | Decimal => {
            let base_type: u32;
            let numbers: &[char];
            if entity_in == Hex {
              base_type = 16;
              // remove the prefix '#x'
              numbers = &entity[2..];
            } else {
              base_type = 10;
              // remove the prefix '#'
              numbers = &entity[1..];
            }
            let numbers = numbers.iter().collect::<String>();
            if let Ok(char_code) = i64::from_str_radix(&numbers, base_type) {
              if (0..=0x10ffff).contains(&char_code) {
                if let Some(last_ch) = std::char::from_u32(char_code as u32) {
                  result.push(last_ch);
                  is_entity_complete = true;
                }
              }
            }
          }
          _ => {
            // entity '&;'
          }
        }
      }
      entity_in = Unkown;
      // wrong entity
      if !is_entity_complete {
        result.push('&');
        result.extend(entity);
        result.push(ch);
        entity = Vec::with_capacity(5);
      } else {
        entity.clear();
        is_in_entity = false;
      }
    }
  }
  result
}

/// Decode a html code's entities into unicode characters, include the `Decimal` `Hex` `Named`.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let content = "<";
/// assert_eq!(decode("&lt;"), content);
/// assert_eq!(decode("&#60;"), content);
/// assert_eq!(decode("&#x3c;"), content);
/// ```
pub fn decode(content: &str) -> String {
  let chars: Vec<char> = content.chars().collect();
  decode_chars(chars).into_iter().collect::<String>()
}

#[derive(Default)]
pub struct Entity {
  entity_in: Option<EntityIn>,
  pub characters: Vec<char>,
  pub entity_type: Option<EncodeType>,
  pub is_end: bool,
}

impl Entity {
  pub fn new() -> Self {
    Entity::default()
  }
  pub fn add(&mut self, ch: char) -> bool {
    if self.is_end {
      return false;
    }
    use EntityIn::*;
    if let Some(entity_in) = &self.entity_in {
      let mut is_in_entity = true;
      match entity_in {
        Named => {
          if !ch.is_ascii_alphabetic() {
            is_in_entity = false;
          }
        }
        Hex | Decimal => match ch {
          '0'..='9' => {}
          'a'..='f' | 'A'..='F' if entity_in == &Hex => {}
          _ => {
            is_in_entity = false;
          }
        },
        Unkown => {
          if ch.is_ascii_alphabetic() {
            self.entity_in = Some(Named);
          } else if ch == '#' {
            self.entity_in = Some(HexOrDecimal);
          } else {
            is_in_entity = false;
          }
        }
        HexOrDecimal => match ch {
          '0'..='9' => {
            self.entity_in = Some(Decimal);
          }
          'x' | 'X' => {
            self.entity_in = Some(Hex);
          }
          _ => {
            is_in_entity = false;
          }
        },
      };
      if is_in_entity {
        self.characters.push(ch);
      }
      return false;
    }
    if ch == '&' {
      self.characters.push(ch);
      self.entity_in = Some(Unkown);
      return true;
    }
    false
  }
}
