/**
encode时：按照二分算法查找根据字符的code值判断是否需要encode，然后挑选最靠前的一个
decode时：复制一份的数据，按照归并排序将数据排序为("&xxx", 0xxx, 'x', len)的元组数组
用HashMap<(char, bool), usize>：存取找到的首字母开始和结束索引，缩小查找范围
查找的时候，先找到首字母的首位端，再二分查找，根据len，第二个字母的charCode
*/
use crate::data::{EntityItem, ENTITIES};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

type SortedEntity = Vec<EntityItem>;
type Positions = HashMap<u8, (usize, usize)>;

lazy_static! {
  static ref IS_SORTED: Mutex<bool> = Mutex::new(false);
  static ref DECODE_ENTITIES: Mutex<SortedEntity> = Mutex::new(vec![]);
  static ref FIRST_POSITION: Mutex<Positions> = Mutex::new(HashMap::new());
}

/*
#[derive(Debug, Eq)]
struct DecodeEntityItem {
  chars: Vec<u8>,
  code: u32,
}
impl Ord for DecodeEntityItem {
  fn cmp(&self, other: &Self) -> Ordering {
    let Self { chars, .. } = self;
    let Self {
      chars: other_chars, ..
    } = other;
    let cur_len = chars.len();
    let other_len = other_chars.len();
    let max_len = if cur_len > other_len {
      cur_len
    } else {
      other_len
    };
    for index in 0..max_len {
      let cur_char = chars.get(index).unwrap_or(&0);
      let other_char = other_chars.get(index).unwrap_or(&0);
      if cur_char != other_char {
        return cur_char.cmp(other_char);
      }
    }
    Ordering::Equal
  }
}
impl PartialOrd for DecodeEntityItem {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for DecodeEntityItem {
  fn eq(&self, other: &Self) -> bool {
    let Self { chars, .. } = self;
    let Self {
      chars: other_chars, ..
    } = other;
    let len = chars.len();
    if len == other_chars.len() {
      for index in 0..len {
        if chars[index] != other_chars[index] {
          return false;
        }
      }
      return true;
    }
    false
  }
}
*/

/**
 * Find the charCode index
 * 二分查找法查找当前code
*/
/*
fn search_char_code_pos(char_code: u32, start_index: usize, end_index: usize) -> Option<usize> {
  let (index, finded) = binary_search_index(char_code, start_index, end_index, |i| {
    let (_, code) = &ENTITIES[i];
    *code
  });
  if finded {
    Some(index)
  } else {
    None
  }
}

fn binary_search_index<T: Ord + Debug, F>(
  compared: T,
  start_index: usize,
  end_index: usize,
  cb: F,
) -> (usize, bool)
where
  F: Fn(usize) -> T,
{
  let start_value = cb(start_index);
  let end_value = cb(end_index);
  let more_than_start = compared > start_value;
  if more_than_start && compared < end_value {
    let middle_index = (start_index + end_index) / 2;
    let middle_value = cb(middle_index);
    if compared == middle_value {
      return (middle_index, true);
    }
    if compared > middle_value {
      if end_index - middle_index > 1 {
        return binary_search_index(compared, middle_index, end_index, cb);
      }
      return (end_index, false);
    } else {
      if middle_index - start_index > 1 {
        return binary_search_index(compared, start_index, middle_index, cb);
      }
      return (start_index + 1, false);
    }
  }
  if compared == start_value {
    return (start_index, true);
  }
  if compared == end_value {
    return (end_index, true);
  }
  if !more_than_start {
    // less than start
    (start_index, false)
  } else {
    // more than start, but also more than end
    (end_index + 1, false)
  }
}
*/

/**
 * Encode,With replaced count
 * 将字符转化为html entity实体
 */
pub fn encode_with_count(content: &str) -> (String, u32) {
  let mut result = String::with_capacity(content.len() + 5);
  let mut replaced_count: u32 = 0;
  // let last_index = ENTITIES.len() - 1;
  for ch in content.chars() {
    let char_code = ch as u32;
    /*
    if let Some(index) = search_char_code_pos(char_code, 0, last_index)
    */
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
      replaced_count += 1;
    } else {
      result.push(ch);
    }
  }
  (result, replaced_count)
}
/**
 * Encode
*/
pub fn encode(content: &str) -> String {
  let (result, _) = encode_with_count(content);
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
    /*
      let (index, _) = binary_search_index(&cur, 0, len - 1, |i| &sorted[i]);
      prev_index = index;
    */
    let search = cur.0;
    prev_index = match sorted[..].binary_search_by(|&(name, _)| name.cmp(search)) {
      Ok(index) => index,
      Err(index) => index,
    };
  }
  (*sorted).insert(prev_index, cur);
}

#[derive(Debug, PartialEq, Eq)]
enum EntityType {
  Unkown,
  Named,
  Hex,
  Number,
  HexOrNumber,
}
/**
 * Decode Chars
 * 将html实体转化为具体字符
 */
pub fn decode_chars(chars: Vec<char>) -> Vec<char> {
  use EntityType::*;
  let mut is_sorted = IS_SORTED.lock().unwrap();
  if !*is_sorted {
    sort_entities();
    *is_sorted = true;
  }
  let sorted = DECODE_ENTITIES.lock().unwrap();
  let firsts = FIRST_POSITION.lock().unwrap();
  let mut result: Vec<char> = Vec::with_capacity(chars.len());
  let mut entity: Vec<char> = Vec::with_capacity(5);
  let mut is_in_entity: bool = false;
  let mut entity_type: EntityType = Unkown;
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
        match entity_type {
          Named => {
            if !ch.is_ascii_alphabetic() {
              is_in_entity = false;
            }
          }
          Hex | Number => match ch {
            '0'..='9' => {}
            'a'..='f' | 'A'..='F' if entity_type == Hex => {}
            _ => {
              is_in_entity = false;
            }
          },
          Unkown => {
            if ch.is_ascii_alphabetic() {
              entity_type = Named;
            } else if ch == '#' {
              entity_type = HexOrNumber;
            } else {
              is_in_entity = false;
            }
          }
          HexOrNumber => match ch {
            '0'..='9' => {
              entity_type = Number;
            }
            'x' | 'X' => {
              entity_type = Hex;
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
        match entity_type {
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
          Hex | Number => {
            let base_type: u32;
            let numbers: &[char];
            if entity_type == Hex {
              base_type = 16;
              // remove the suffix '#x'
              numbers = &entity[2..];
            } else {
              base_type = 10;
              // remove the suffix '#'
              numbers = &entity[1..];
            }
            let numbers = numbers.iter().collect::<String>();
            if let Ok(char_code) = i64::from_str_radix(&numbers, base_type) {
              if char_code >= 0 && char_code <= 0x10ffff {
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
      entity_type = Unkown;
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
/**
 * Decode
 */
pub fn decode(content: &str) -> String {
  let chars: Vec<char> = content.chars().collect();
  decode_chars(chars).into_iter().collect::<String>()
}
