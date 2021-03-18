use crate::data::{EntityItem, ENTITIES};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
type SortedEntity = Vec<EntityItem>;
type Positions = HashMap<u8, (usize, usize)>;

/// NOOP is the None value of Option<dyn Fn(char)->bool>  
pub const NOOP: Option<&dyn Fn(&char) -> bool> = None::<&dyn Fn(&char) -> bool>;
pub type CharEntityHash = HashMap<char, Vec<char>>;

lazy_static! {
	static ref IS_SORTED: AtomicBool = AtomicBool::new(false);
	static ref DECODE_ENTITIES: Mutex<SortedEntity> = Mutex::new(vec![]);
	static ref FIRST_POSITION: Mutex<Positions> = Mutex::new(HashMap::new());
	// special chars
	static ref HTML_CHARS:  CharEntityHash = {
		let mut map = HashMap::with_capacity(3);
		map.insert('>', vec!['&', 'g', 't', ';']);
		map.insert('<', vec!['&', 'l', 't', ';']);
		map.insert('&', vec!['&', 'a', 'm', 'p', ';']);
		map
	};
	static ref SPECIAL_CHARS: CharEntityHash = {
		let mut map = HashMap::with_capacity(5);
		map.insert('"', vec!['&', 'q', 'u', 'o', 't', ';']);
		map.insert('\'', vec!['&', 'a', 'p', 'o', 's', ';']);
		for (k, v) in HTML_CHARS.iter(){
				map.insert(*k, v.clone());
		}
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
/// let char_encoded = encode_char(&character, &EncodeType::Named, &NOOP).iter().collect::<String>();
/// assert_eq!(char_encoded, "&lt;");
///
/// let character = '<';
/// let char_encoded = encode_char(&character, &EncodeType::Decimal, &NOOP).iter().collect::<String>();
/// assert_eq!(char_encoded, "&#60;");
///
/// let character = '<';
/// let char_encoded = encode_char(&character, &EncodeType::Hex, &NOOP).iter().collect::<String>();
/// assert_eq!(char_encoded, "&#x3c;");
///
/// let character = '<';
/// let char_encoded = encode_char(&character, &EncodeType::Named, &Some(|ch:&char|*ch == '<')).iter().collect::<String>();
/// assert_eq!(char_encoded, "<");
/// ```
pub fn encode_char<F>(ch: &char, encode_type: &EncodeType, exclude_fn: &Option<F>) -> Vec<char>
where
	F: Fn(&char) -> bool,
{
	use EncodeType::*;
	let encode_type = *encode_type as u8;
	let ch = *ch;
	let char_code = ch as u32;

	if encode_type & (Named as u8) > 0 {
		let mut should_find_name = true;
		if let Some(exclude_fn) = exclude_fn {
			if exclude_fn(&ch) {
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
				let mut result = vec!['&'];
				result.extend(entity.chars());
				result.push(';');
				return result;
			}
		}
	}
	if encode_type & (Hex as u8) > 0 {
		let mut result = vec!['&', '#', 'x'];
		let hex = format!("{:x}", char_code);
		result.extend(hex.chars());
		result.push(';');
		return result;
	}
	if encode_type & (Decimal as u8) > 0 {
		let mut result = vec!['&', '#'];
		result.extend(char_code.to_string().chars());
		result.push(';');
		return result;
	}
	vec![ch]
}

fn filter_entity_set(
	charset: &CharEntityHash,
	encode_type: &EncodeType,
	ch: &char,
) -> (bool, Option<Vec<char>>) {
	let encode_type = *encode_type as u8;
	if let Some(v) = charset.get(ch) {
		if (encode_type & EncodeType::Named as u8) > 0 {
			return (true, Some(v.clone()));
		}
		return (true, None);
	}
	(false, None)
}

/// The type of characters you need encoded, default: `SpecialCharsAndNoASCII`
pub enum EntitySet {
	Empty = 0,
	/// encode all
	All = 1,
	/// encode character not ascii                 
	NoASCII = 2,
	/// encode '<','>','&', main for entity in text node when call element's `innerHtml()` method                
	Html = 3,
	/// encode '<','>','&', '\'', '"'                
	SpecialChars = 4,
	/// this is default
	SpecialCharsAndNoASCII = 6,
}

impl Default for EntitySet {
	fn default() -> Self {
		EntitySet::SpecialCharsAndNoASCII
	}
}

impl EntitySet {
	/// check if a character need encode by the encode type, and encode it if nessessary.
	pub fn filter(&self, ch: &char, encode_type: &EncodeType) -> (bool, Option<Vec<char>>) {
		use EntitySet::*;
		match self {
			SpecialChars => filter_entity_set(&SPECIAL_CHARS, encode_type, ch),
			Html => filter_entity_set(&HTML_CHARS, encode_type, ch),
			NoASCII => (*ch as u32 > 0x80, None),
			SpecialCharsAndNoASCII => {
				let (need_encode, result) = EntitySet::NoASCII.filter(ch, encode_type);
				if need_encode {
					return (need_encode, result);
				}
				EntitySet::SpecialChars.filter(ch, encode_type)
			}
			All => (true, None),
			Empty => (false, None),
		}
	}
	/// check if the set contains the character.
	pub fn contains(&self, ch: &char) -> bool {
		let (flag, _) = self.filter(ch, &EncodeType::Decimal);
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
/// let html_encoded = encode(html, EntitySet::SpecialChars, EncodeType::Named);
/// assert_eq!(html_encoded.iter().collect::<String>(), "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
///
/// let html_decoded = decode_chars(&html_encoded);
/// assert_eq!(html, html_decoded.iter().collect::<String>());
/// ```
pub fn encode(content: &str, entity_set: EntitySet, encode_type: EncodeType) -> Vec<char> {
	let mut result = Vec::with_capacity(content.len() + 5);
	for ch in content.chars() {
		let (need_encode, encoded) = entity_set.filter(&ch, &encode_type);
		if need_encode {
			if let Some(encoded) = encoded {
				result.extend_from_slice(&encoded[..]);
			} else {
				let encoded = encode_char(&ch, &encode_type, &NOOP);
				result.extend_from_slice(&encoded[..]);
			}
		} else {
			result.push(ch);
		}
	}
	result
}

/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let html = "<div class='header'></div>";
/// let html_encoded = encode_chars(&html.chars().collect::<Vec<char>>(), EntitySet::SpecialChars, EncodeType::Named);
/// assert_eq!(html_encoded.iter().collect::<String>(), "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
///
/// let html_decoded = decode_chars(&html_encoded);
/// assert_eq!(html, html_decoded.iter().collect::<String>());
/// ```
pub fn encode_chars(content: &[char], entity_set: EntitySet, encode_type: EncodeType) -> Vec<char> {
	let mut result = Vec::with_capacity(content.len() + 5);
	for ch in content {
		let (need_encode, encoded) = entity_set.filter(ch, &encode_type);
		if need_encode {
			if let Some(encoded) = encoded {
				result.extend_from_slice(&encoded[..]);
			} else {
				let encoded = encode_char(ch, &encode_type, &NOOP);
				result.extend_from_slice(&encoded[..]);
			}
		} else {
			result.push(*ch);
		}
	}
	result
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
/// let html_encoded = encode_filter(&html.chars().collect::<Vec<char>>(), |ch: &char|{
///   // special characters but not '<'
///   *ch != '<' && EntitySet::SpecialChars.contains(ch)
/// }, EncodeType::Named, NOOP);
/// assert_eq!(html_encoded.iter().collect::<String>(), "<div class=&apos;header&apos;&gt;</div&gt;");
///
/// // special characters, but exclude the single quote "'" use named.
/// let html = "<div class='header'></div>";
/// let html_encoded = encode_filter(&html.chars().collect::<Vec<char>>(), |ch: &char|{
///   EntitySet::SpecialChars.contains(ch)
/// }, EncodeType::NamedOrDecimal, Some(|ch: &char| *ch == '\''));
/// assert_eq!(html_encoded.iter().collect::<String>(), "&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;");
/// ```
pub fn encode_filter<F: Fn(&char) -> bool, C: Fn(&char) -> bool>(
	content: &[char],
	filter_fn: F,
	encode_type: EncodeType,
	exclude_fn: Option<C>,
) -> Vec<char> {
	let mut result: Vec<char> = Vec::with_capacity(content.len() + 5);
	for ch in content {
		if filter_fn(ch) {
			result.extend_from_slice(&encode_char(ch, &encode_type, &exclude_fn.as_ref()));
		} else {
			result.push(*ch);
		}
	}
	result
}

/// encode with the Encoder function.
///
/// # Examples
/// ```
/// use htmlentity::entity::*;
///
/// let html = "<div class='header'></div>";
/// let html_encoded = encode_with(&html.chars().collect::<Vec<char>>(), |ch:&char|{
///   if(EntitySet::SpecialChars.contains(ch)){
///     return Some(EncodeType::Named);
///   }
///   None
/// });
/// assert_eq!(html_encoded.iter().collect::<String>(), "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");
///
/// let html_decoded = decode_chars(&html_encoded);
/// ```
pub fn encode_with<F>(content: &[char], encoder: F) -> Vec<char>
where
	F: Fn(&char) -> Option<EncodeType>,
{
	let mut result: Vec<char> = Vec::with_capacity(content.len() + 5);
	for ch in content {
		if let Some(encode_type) = encoder(ch) {
			result.extend_from_slice(&encode_char(ch, &encode_type, &NOOP));
		} else {
			result.push(*ch);
		}
	}
	result
}
/**
 * Sort
 */
fn sort_entities() {
	let mut sorted: SortedEntity = Vec::with_capacity(ENTITIES.len());
	let mut counts: Positions = HashMap::new();
	let mut firsts: Vec<u8> = Vec::with_capacity(52);
	// binary search
	for pair in &ENTITIES[..] {
		let entity = *pair;
		let chars = entity.0.as_bytes();
		let first = chars[0];
		binary_insert(&mut sorted, entity);
		// save the first character index to HashMap
		match counts.get_mut(&first) {
			Some((v, _)) => {
				*v += 1;
			}
			None => {
				counts.insert(first, (1, 0));
			}
		}
		// insert
		if !firsts.contains(&first) {
			firsts.push(first);
		}
	}
	// sort
	firsts.sort_unstable();
	let mut cur_index: usize = 0;
	for char_code in firsts {
		let position = counts.get_mut(&char_code).unwrap();
		let next_index = cur_index + position.0;
		*position = (cur_index, next_index);
		cur_index = next_index;
	}
	// save index to positions
	let mut positions = FIRST_POSITION.lock().unwrap();
	*positions = counts;
	// save sorted entities
	let mut entities = DECODE_ENTITIES.lock().unwrap();
	*entities = sorted;
}
/**
 * binary insert
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
#[derive(Copy, Clone)]
pub enum EncodeType {
	Ignore = 0,
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
/// assert_eq!(decode_chars(&"&lt;".chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars(&"&#60;".chars().collect::<Vec<char>>()), char_list);
/// assert_eq!(decode_chars(&"&#x3c;".chars().collect::<Vec<char>>()), char_list);
/// ```
pub fn decode_chars(chars: &[char]) -> Vec<char> {
	let mut decoder = DecoderBuilder::new(chars.len());
	for ch in chars {
		decoder.add(*ch);
	}
	decoder.eof();
	decoder.data()
}

/// Decode a html code's entities into unicode characters, include the `Decimal` `Hex` `Named`.
///
/// # Examples
///
/// ```
/// use htmlentity::entity::*;
///
/// let content = "<";
/// assert_eq!(decode("&lt;").iter().collect::<String>(), content);
/// assert_eq!(decode("&#60;").iter().collect::<String>(), content);
/// assert_eq!(decode("&#x3c;").iter().collect::<String>(), content);
/// ```
pub fn decode(content: &str) -> Vec<char> {
	let chars = content.chars();
	let mut decoder = DecoderBuilder::new(content.len());
	for ch in chars {
		decoder.add(ch);
	}
	decoder.eof();
	decoder.data()
}
/// Entity struct
#[derive(Default)]
pub struct Entity {
	pub entity_in: Option<EntityIn>,
	pub characters: Vec<char>,
	pub is_end: bool,
}

impl Entity {
	/// Return an Entity struct, same as Entity::default()
	pub fn new() -> Self {
		Entity::default()
	}
	/// `add(ch: char)`: check if the character is an allowed character
	pub fn add(&mut self, ch: char) -> bool {
		if self.is_end {
			return false;
		}
		use EntityIn::*;
		if let Some(entity_in) = &self.entity_in {
			let mut is_in_entity = true;
			if ch == ';' {
				self.is_end = true;
				return true;
			} else {
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
				return is_in_entity;
			}
		} else if ch == '&' {
			self.entity_in = Some(Unkown);
			return true;
		}
		false
	}
	/// `decode()`: decode the entity, if ok, return the unicode character.
	pub fn decode(&self) -> Option<char> {
		if !self.is_end {
			return None;
		}
		use EntityIn::*;
		let entity = &self.characters;
		let entity_in = self.entity_in.as_ref().unwrap();
		match entity_in {
			Named => {
				// try to find the entity
				let first = entity[0] as u32 as u8;
				// sort the named characters
				let is_sorted = IS_SORTED.load(Ordering::SeqCst);
				if !is_sorted {
					sort_entities();
					IS_SORTED.store(true, Ordering::SeqCst);
				}
				let sorted = DECODE_ENTITIES.lock().unwrap();
				let firsts = FIRST_POSITION.lock().unwrap();
				if let Some(&(start_index, end_index)) = firsts.get(&first) {
					let searched = entity.iter().collect::<String>();
					if let Ok(find_index) = sorted[start_index..end_index]
						.binary_search_by(|&(name, _)| name.cmp(searched.as_str()))
					{
						let last_index = start_index + find_index;
						let (_, code) = sorted[last_index];
						return Some(std::char::from_u32(code).unwrap());
					}
				}
			}
			Hex | Decimal => {
				let base_type: u32;
				let numbers: &[char];
				if entity_in == &Hex {
					base_type = 16;
					// remove the prefix '#x'
					numbers = &entity[2..];
				} else {
					base_type = 10;
					// remove the prefix '#'
					numbers = &entity[1..];
				}
				if numbers.is_empty() {
					// '&#;' '&#x;'
					return None;
				}
				let numbers = numbers.iter().collect::<String>();
				if let Ok(char_code) = i64::from_str_radix(&numbers, base_type) {
					if (0..=0x10ffff).contains(&char_code) {
						if let Some(last_ch) = std::char::from_u32(char_code as u32) {
							return Some(last_ch);
						}
					}
				}
			}
			_ => {
				// entity '&;' '&#'
			}
		}
		None
	}
	/// `get_chars()` return the characters of the entity,if it's a correct entity, it will return the Vec with the decoded unicode character, otherwise return all the characters.
	pub fn get_chars(&self) -> Vec<char> {
		if let Some(ch) = self.decode() {
			return vec![ch];
		}
		let is_end = self.is_end;
		let mut result = Vec::with_capacity(self.characters.len() + 1 + is_end as usize);
		result.push('&');
		result.extend(&self.characters);
		if is_end {
			result.push(';');
		}
		result
	}
}

struct DecoderBuilder {
	entity: Entity,
	data: Vec<char>,
	is_in_entity: bool,
}

impl DecoderBuilder {
	//	create a new decoder
	fn new(total: usize) -> Self {
		let data: Vec<char> = Vec::with_capacity(total);
		let entity: Entity = Entity::new();
		let is_in_entity: bool = false;
		DecoderBuilder {
			entity,
			data,
			is_in_entity,
		}
	}
	// add a character
	fn add(&mut self, ch: char) {
		if !self.is_in_entity {
			if self.entity.add(ch) {
				self.is_in_entity = true;
			} else {
				self.data.push(ch);
			}
		} else {
			let is_wrong_entity = !self.entity.add(ch);
			if is_wrong_entity || self.entity.is_end {
				let chars = self.entity.get_chars();
				self.data.extend(chars);
				if is_wrong_entity {
					self.data.push(ch);
				}
				self.is_in_entity = false;
				self.entity = Entity::new();
			}
		}
	}
	// end the chars
	fn eof(&mut self) {
		if self.is_in_entity {
			let chars = self.entity.get_chars();
			self.data.extend(chars);
		}
	}
	// return the data
	fn data(self) -> Vec<char> {
		self.data
	}
}
