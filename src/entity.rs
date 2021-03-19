use crate::data::{ENTITIES, FIRST_LETTER_POSITION, LETTER_ORDERED_ENTITIES};
use lazy_static::lazy_static;
use std::{char, collections::HashMap};

/// NOOP is the None value of Option<dyn Fn(char)->bool>  
pub const NOOP: Option<&dyn Fn(&char) -> bool> = None::<&dyn Fn(&char) -> bool>;
pub type CharEntityHash = HashMap<char, Vec<char>>;

lazy_static! {
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
	// normal name entity
	static ref NORMAL_NAME_ENTITY_CHAR: HashMap<Vec<char>, char> = {
		let mut map = HashMap::with_capacity(10);
		map.insert(vec!['l', 't'], '<');
		map.insert(vec!['L', 'T'], '<');
		map.insert(vec!['g', 't'], '>');
		map.insert(vec!['G', 'T'], '>');
		map.insert(vec!['a', 'm', 'p'], '&');
		map.insert(vec!['A', 'M', 'P'], '&');
		map.insert(vec!['q', 'u', 'o', 't'], '"');
		map.insert(vec!['Q', 'U', 'O', 'T'], '"');
		map.insert(vec!['a', 'p', 'o', 's'], '\'');
		map.insert(vec!['n', 'b', 's', 'p'], char::from_u32(0xa0).expect("0xa0 is a &nbsp; entity"));
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
			let finded = ENTITIES.binary_search_by_key(&char_code, |&(_, code)| code);
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
				let (entity, _) = &ENTITIES[first_index];
				let mut result = vec!['&'];
				result.extend(entity.chars().into_iter());
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

#[derive(PartialEq, Eq)]
pub enum EntityType {
	Named,
	Hex,
	Decimal,
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
	let mut data: Vec<char> = Vec::with_capacity(chars.len());
	decode_chars_to(chars, &mut data);
	data
}
/// decode chars to
pub fn decode_chars_to(chars: &[char], data: &mut Vec<char>) {
	let mut is_in_entity = false;
	let mut start_index: usize = 0;
	for (index, ch) in chars.iter().enumerate() {
		if !is_in_entity {
			if ch == &'&' {
				start_index = index;
				is_in_entity = true;
			} else {
				data.push(*ch);
			}
		} else if ch == &';' {
			// entity end
			let entity = &chars[start_index + 1..index];
			if let Some(ch) = Entity::decode(&entity) {
				// entity ok
				data.push(ch);
			} else {
				// wrong entity
				data.push('&');
				data.extend_from_slice(entity);
				data.push(';');
			}
			is_in_entity = false;
		}
	}
	// wrong entity at the end
	if is_in_entity {
		data.extend(&chars[start_index..]);
	}
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
	let total = content.len();
	let mut data: Vec<char> = Vec::with_capacity(total);
	let mut entity: Vec<char> = Vec::with_capacity(5);
	let mut is_in_entity = false;
	for ch in content.chars() {
		if !is_in_entity {
			if ch == '&' {
				is_in_entity = true;
			} else {
				data.push(ch);
			}
		} else {
			// in entity
			if ch == ';' {
				if let Some(decode_char) = Entity::decode(&entity) {
					data.push(decode_char);
					entity.clear();
				} else {
					data.push('&');
					data.append(&mut entity);
					data.push(';');
				}
				is_in_entity = false;
			} else {
				entity.push(ch);
			}
		}
	}
	// wrong entity at the end
	if is_in_entity {
		data.push('&');
		data.extend(entity);
	}
	data
}
/// decode a content and append to the data string
pub fn decode_to(content: &str, data: &mut String) {
	let mut entity: Vec<char> = Vec::with_capacity(5);
	let mut is_in_entity = false;
	for ch in content.chars() {
		if !is_in_entity {
			if ch == '&' {
				is_in_entity = true;
			} else {
				data.push(ch);
			}
		} else {
			// in entity
			if ch == ';' {
				if let Some(decode_char) = Entity::decode(&entity) {
					data.push(decode_char);
					entity.clear();
				} else {
					data.push('&');
					data.extend(entity);
					data.push(';');
					entity = Vec::with_capacity(5);
				}
				is_in_entity = false;
			} else {
				entity.push(ch);
			}
		}
	}
	// wrong entity at the end
	if is_in_entity {
		data.push('&');
		data.extend(entity);
	}
}
/// Entity struct
#[derive(Default)]
pub struct Entity;

impl Entity {
	/// `decode()`: decode the entity, if ok, return the unicode character.
	pub fn decode(chars: &[char]) -> Option<char> {
		let total = chars.len();
		if total == 0 {
			return None;
		}
		// check type
		let first = &chars[0];
		let mut entity_type: EntityType = EntityType::Named;
		if first.is_ascii_alphabetic() {
			for ch in &chars[1..] {
				if !ch.is_ascii_alphabetic() {
					return None;
				}
			}
		} else if first == &'#' && total > 1 {
			let second = chars[1];
			match second {
				'0'..='9' => {
					// decimal
					for ch in &chars[2..] {
						if !matches!(ch, '0'..='9') {
							return None;
						}
					}
					entity_type = EntityType::Decimal;
				}
				'x' | 'X' => {
					// hex
					if total > 2 {
						for ch in &chars[2..] {
							if !matches!(ch, 'a'..='f' | 'A'..='F' | '0'..='9') {
								return None;
							}
						}
						entity_type = EntityType::Hex;
					} else {
						return None;
					}
				}
				_ => {
					return None;
				}
			}
		} else {
			return None;
		}
		//
		use EntityType::*;
		match entity_type {
			Named => {
				// normal entity characters
				if let Some(&ch) = NORMAL_NAME_ENTITY_CHAR.get(chars) {
					return Some(ch);
				}
				// try to find the entity
				let first_letter = &chars[0];
				let search = chars.iter().collect::<String>();
				let search = search.as_str();
				if let Some(&(start_index, end_index)) = FIRST_LETTER_POSITION.get(&first_letter) {
					if let Some(find_index) = LETTER_ORDERED_ENTITIES[start_index..end_index]
						.iter()
						.position(|(name, _)| name == &search)
					{
						let last_index = start_index + find_index;
						let (_, code) = LETTER_ORDERED_ENTITIES[last_index];
						return Some(std::char::from_u32(code).unwrap());
					}
				}
			}
			Hex | Decimal => {
				let base_type: u32;
				let numbers: &[char];
				if entity_type == Hex {
					base_type = 16;
					// remove the prefix '#x'
					numbers = &chars[2..];
				} else {
					base_type = 10;
					// remove the prefix '#'
					numbers = &chars[1..];
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
		}
		None
	}
}
