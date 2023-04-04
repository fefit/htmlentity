use htmlentity::{
	entity::{decode, decode_chars, encode, EncodeType, EntitySet},
	types::Byte,
};
fn main() -> anyhow::Result<()> {
	let content = "abc&gt;ef&a&lt;";
	let bytes: anyhow::Result<Vec<char>> = decode(content.as_bytes()).into();
	let bytes = bytes?;
	println!("{:#?}", bytes);
	Ok(())
}
