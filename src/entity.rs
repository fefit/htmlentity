use crate::{
	data::{ENTITIES, FIRST_LETTER_POSITION, LETTER_ORDERED_ENTITIES},
	types::{Byte, ByteList, BytesCharEntity, EntityCharBytes},
};
use lazy_static::lazy_static;
use std::{borrow::Cow, char, collections::HashMap, ops::RangeInclusive};

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

#[inline]
fn char_to_utf8_bytes(ch: char) -> ByteList {
	let len = ch.len_utf8();
	let mut bytes: ByteList = Vec::with_capacity(len);
	for _ in 0..len {
		bytes.push(0);
	}
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
fn numbers_to_char(bytes: &[Byte], radix: u32) -> Option<char> {
	if !bytes.is_empty() {
		// '&#;' '&#x;'
		let bytes = std::str::from_utf8(bytes).expect("the bytes is not regular utf8 encoding");
		if let Ok(char_code) = i64::from_str_radix(bytes, radix) {
			return std::char::from_u32(char_code as u32);
		}
	}
	None
}

pub trait IEntityTrait {}

#[derive(Debug)]
pub struct DecodeData<'b> {
	bytes: Cow<'b, [Byte]>,
	entities: Vec<(RangeInclusive<usize>, char)>,
}

impl<'b> ToString for DecodeData<'b> {
	fn to_string(&self) -> String {
		if self.entities.is_empty() {
			return std::str::from_utf8(&self.bytes)
				.expect("decode data has incorrect utf8 byte sequence.")
				.to_string();
		}
		let mut result = String::with_capacity(self.bytes.len());
		let mut index = 0;
		result
	}
}

#[derive(Debug)]
pub struct EncodeData<'b> {
	bytes: Cow<'b, [Byte]>,
	entities: Vec<(RangeInclusive<usize>, CharEntity)>,
}

/// EncodeType: the output format type, default: `NamedOrDecimal`
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

fn filter_entity_set(
	charset: &EntityCharBytes,
	encode_type: &EncodeType,
	ch: &char,
) -> (bool, Option<Cow<'static, [Byte]>>) {
	let encode_type = *encode_type as u8;
	if let Some(&v) = charset.get(ch) {
		if (encode_type & EncodeType::Named as u8) > 0 {
			return (true, Some(Cow::from(v)));
		}
		return (true, None);
	}
	(false, None)
}

/// The type of characters you need encoded, default: `SpecialCharsAndNoASCII`
#[derive(Default)]
pub enum EntitySet {
	/// encode all
	All = 1,
	/// encode character not ascii                 
	NoASCII = 2,
	/// encode '<','>','&', main for entity in text node when call element's `innerHtml()` method    
	#[default]
	Html = 3,
	/// encode '<','>','&', '\'', '"'                
	SpecialChars = 4,
	/// this is default
	SpecialCharsAndNoASCII = 6,
}

impl EntitySet {
	/// check if a character need encode by the encode type, and encode it if nessessary.
	pub fn filter(
		&self,
		ch: &char,
		encode_type: &EncodeType,
	) -> (bool, Option<Cow<'static, [Byte]>>) {
		use EntitySet::*;
		match self {
			SpecialChars => filter_entity_set(&SPECIAL_BYTES, encode_type, ch),
			Html => filter_entity_set(&HTML_BYTES, encode_type, ch),
			NoASCII => (*ch as u32 > 0x80, None),
			SpecialCharsAndNoASCII => {
				let result = EntitySet::NoASCII.filter(ch, encode_type);
				if result.0 {
					return result;
				}
				EntitySet::SpecialChars.filter(ch, encode_type)
			}
			All => (true, None),
		}
	}
	/// check if the set contains the character.
	pub fn contains(&self, ch: &char) -> bool {
		let (flag, _) = self.filter(ch, &EncodeType::Decimal);
		flag
	}
}

#[derive(PartialEq, Eq, Debug)]
pub enum EntityType {
	Named,
	Hex,
	Decimal,
}

#[derive(Debug)]
pub struct CharEntity {
	entity_type: EntityType,
	entity_data: Cow<'static, [Byte]>,
}

/// Entity struct
#[derive(Default)]
pub struct Entity;

impl Entity {
	/// `decode()`: decode the entity, if ok, return the unicode character.
	pub fn decode(bytes: &[Byte]) -> Option<char> {
		let total = bytes.len();
		if total == 0 {
			return None;
		}
		// check type
		let first = bytes[0];
		let mut entity_type: EntityType = EntityType::Named;
		if first.is_ascii_alphabetic() {
			for ch in &bytes[1..] {
				if !ch.is_ascii_alphabetic() {
					return None;
				}
			}
		} else if first == b'#' && total > 1 {
			let second = bytes[1];
			match second {
				b'0'..=b'9' => {
					// decimal
					for byte in &bytes[2..] {
						if byte.is_ascii_digit() {
							return None;
						}
					}
					entity_type = EntityType::Decimal;
				}
				b'x' | b'X' => {
					// hex
					if total > 2 {
						for byte in &bytes[2..] {
							if !matches!(byte, b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9') {
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
		// go on the steps
		match entity_type {
			// named entity
			EntityType::Named => {
				// normal entity characters
				if let Some(&ch) = NORMAL_NAME_ENTITY_BYTE.get(bytes) {
					return Some(ch);
				}
				// try to find the entity
				if let Some(&(start_index, end_index)) = FIRST_LETTER_POSITION.get(&bytes[0]) {
					if let Some(find_index) = LETTER_ORDERED_ENTITIES[start_index..end_index]
						.iter()
						.position(|&(name, _)| name == bytes)
					{
						let last_index = start_index + find_index;
						let (_, code) = LETTER_ORDERED_ENTITIES[last_index];
						return Some(code);
					}
				}
			}
			// hex entity
			EntityType::Hex => {
				// remove the prefix '#x'
				return numbers_to_char(&bytes[2..], 16);
			}
			// decimal entity
			EntityType::Decimal => {
				// remove the prefix '#'
				return numbers_to_char(&bytes[1..], 10);
			}
		}
		None
	}
}

/**
 * Encode character to entity bytes
 */
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

pub fn encode<'a>(
	content: &'a [Byte],
	encode_type: &EncodeType,
	charset: &EntitySet,
) -> EncodeData<'a> {
	encode_with(content, encode_type, |ch, encode_type| {
		charset.filter(ch, encode_type)
	})
}
/**
 *
 */
pub fn encode_with<'a>(
	content: &'a [Byte],
	encode_type: &EncodeType,
	filter_fn: impl Fn(&char, &EncodeType) -> (bool, Option<Cow<'static, [Byte]>>),
) -> EncodeData<'a> {
	let mut ch: u32 = 0;
	let mut last_ch: char = '\0';
	let mut next_count = 0;
	let mut start_index: usize = 0;
	let mut entities: Vec<(RangeInclusive<usize>, CharEntity)> = vec![];
	for (idx, byte) in content.iter().enumerate() {
		match next_count {
			0 => {
				start_index = idx;
				if (byte >> 7) == 0 {
					// entity
					last_ch = *byte as char;
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
								// wrong utf8, ignore
								next_count = 0;
							}
						}
					}
					// jump the next steps
					continue;
				}
			}
			1 | 2 | 3 => {
				if (byte >> 6) == 0b10 {
					next_count -= 1;
					ch += ((byte & 0b111111) as u32) << (next_count * 6);
					if next_count == 0 {
						last_ch = char::from_u32(ch).expect(
							"An incorrect ut8 byte sequence was encountered when calling the 'encode' method",
						);
					} else {
						continue;
					}
				} else {
					// wrong utf8
					next_count = 0;
					continue;
				}
			}
			// unreachable feature
			_ => unreachable!(),
		};
		let (need_encode, maybe_entity) = filter_fn(&last_ch, encode_type);
		if need_encode {
			if let Some(entity) = maybe_entity {
				entities.push((
					start_index..=idx,
					CharEntity {
						entity_type: EntityType::Named,
						entity_data: entity,
					},
				));
			} else if let Some(entity) = encode_char(&last_ch, encode_type) {
				entities.push((start_index..=idx, entity));
			}
		}
	}
	EncodeData {
		bytes: Cow::from(content),
		entities,
	}
}

/**
 * decode chars into Cow chars
 */
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
			if *ch == ';' {
				// end of the entity, ignore '&;'
				if start_index != idx {
					let bytes = tr_chars_to_utf8_bytes(&chars[start_index..idx]);
					if let Some(bytes) = bytes {
						if let Some(decode_char) = Entity::decode(&bytes) {
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
		}
	}
	Cow::from(chars)
}

/**
 * decode chars into vec chars
 */
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
			if ch == ';' {
				// end of the entity, ignore '&;'
				if start_index != idx {
					let bytes = tr_chars_to_utf8_bytes(&chars[start_index..idx]);
					if let Some(bytes) = bytes {
						if let Some(decode_char) = Entity::decode(&bytes) {
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
		}
	}
	if is_in_entity {
		// add the end non regular entity
		data.extend_from_slice(&chars[start_index - 1..])
	}
}
/**
 * decode bytes to data
 */
pub fn decode(content: &[Byte]) -> DecodeData<'_> {
	let mut entities: Vec<(RangeInclusive<usize>, char)> = vec![];
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
			if *byte == b';' {
				// end of the entity, ignore '&;'
				if start_index != idx {
					if let Some(decode_char) = Entity::decode(&content[start_index..idx]) {
						entities.push((start_index - 1..=idx, decode_char));
					}
				}
				is_in_entity = false;
			}
		}
	}
	// wrong entity at the end
	DecodeData {
		bytes: Cow::from(content),
		entities,
	}
}
