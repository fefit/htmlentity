use htmlentity::entity::{decode, decode_chars, encode, EncodeType, EntitySet};
fn main() {
	let content = "abc><def&a";
	println!(
		"{:#?}",
		encode(&content.as_bytes(), &EncodeType::Named, &EntitySet::Html)
	);
}
