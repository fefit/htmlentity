use htmlentity::{
	entity::{decode, decode_chars, encode, EncodeType, EntitySet, IHtmlEntityTrait},
	types::{Byte, StringResult},
};
fn main() -> anyhow::Result<()> {
	let content = "abc&#x2324;def&lt;gh";
	let bytes = decode(content.as_bytes());
	for byte in &bytes {
		println!("byte:{:?}", byte);
	}
	println!("{:?}", bytes.to_string());
	let bytes = bytes.to_bytes();
	let content = encode(
		&bytes,
		&EncodeType::NamedOrHex,
		&EntitySet::SpecialCharsAndNoASCII,
	);
	println!("{:?}", content.to_string());
	for byte in &content {
		println!("解密:{:?}", byte);
	}
	Ok(())
}
