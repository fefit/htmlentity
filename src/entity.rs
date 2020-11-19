/**
encode时：按照二分算法查找根据字符的code值判断是否需要encode，然后挑选最靠前的一个
decode时：复制一份的数据，按照归并排序将数据排序为("&xxx", 0xxx, 'x', len)的元组数组
用HashMap<(char, bool), usize>：存取找到的首字母开始和结束索引，缩小查找范围
查找的时候，先找到首字母的首位端，再二分查找，根据len，第二个字母的charCode
*/
use crate::data::ENTITIES;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
type SortedEntity = Vec<(Vec<u8>, u32)>;
lazy_static! {
  static ref IS_SORTED: Mutex<bool> = Mutex::new(false);
  static ref DECODE_ENTITIES: Mutex<SortedEntity> = Mutex::new(vec![]);
}
/**
 * Find the charCode index
 * 二分查找法查找当前code
*/
fn search_char_code_pos(char_code: u32, start_index: usize, end_index: usize) -> Option<usize> {
  let (_, start_char_code) = ENTITIES[start_index];
  let (_, end_char_code) = ENTITIES[end_index];
  if char_code > start_char_code && char_code < end_char_code {
    let middle_index = (start_index + end_index) / 2;
    let (_, middle_char_code) = ENTITIES[middle_index];
    if char_code == middle_char_code {
      Some(middle_index)
    } else {
      if char_code > middle_char_code {
        if end_index - middle_index > 1 {
          search_char_code_pos(char_code, middle_index, end_index)
        } else {
          None
        }
      } else {
        if middle_index - start_index > 1 {
          search_char_code_pos(char_code, start_index, middle_index)
        } else {
          None
        }
      }
    }
  } else if char_code == start_char_code {
    Some(start_index)
  } else if char_code == end_char_code {
    Some(end_index)
  } else {
    None
  }
}
/**
 * Encode
 * 将字符转化为html entity实体
 */
pub fn encode(content: &str) -> (String, u32) {
  let last_index = ENTITIES.len() - 1;
  let mut result = String::with_capacity(content.len() + 5);
  let mut replaced_count: u32 = 0;
  for ch in content.chars() {
    let char_code = ch as u32;
    if let Some(index) = search_char_code_pos(char_code, 0, last_index) {
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
      result.push_str(entity);
      replaced_count += 1;
    } else {
      result.push(ch);
    }
  }
  (result, replaced_count)
}
/**
 * Sort
 * 将entities排序成所需格式
 */
fn sort_entities() {
  let result: SortedEntity = Vec::with_capacity(ENTITIES.len());
  let counts: HashMap<u8, u32> = HashMap::new();
  let firsts: Vec<u8> = Vec::with_capacity(52);
  // 二分查找插入
  for pair in &ENTITIES[..] {
    let (entity, code) = pair;
    let entity = entity.as_bytes();
    let entity = (&entity[1..entity.len() - 1]).to_vec();
    let first = entity[0];
  }
}
/**
 * 二分查找插入
 */
fn binary_insert() {}
/**
 * Decode
 * 将html实体转化为具体字符
 */
pub fn decode(content: &str) {
  let mut is_sorted = IS_SORTED.lock().unwrap();
  if !*is_sorted {
    sort_entities();
    *is_sorted = true;
  }
}
