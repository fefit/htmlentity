use htmlentity::{
  entity::{
    decode, decode_chars, decode_to, encode, encode_to, CharacterSet, EncodeType, IBytesTrait,
    ICodedDataTrait,
  },
  types::{Byte, ByteList, StringResult},
};
fn main() -> anyhow::Result<()> {
  let content = "abc&#x2324;def&lt;gh><";
  let bytes = decode(content.as_bytes());
  for byte in &bytes {
    println!("byte:{:?}", byte);
  }
  println!("{:?}", bytes.to_string());

  let bytes = bytes.to_bytes();
  println!("内容：{:?}", std::str::from_utf8(&bytes));
  let content = encode(
    &bytes,
    &EncodeType::NamedOrHex,
    &CharacterSet::SpecialCharsAndNoASCII,
  );
  println!("{:?}", content.to_string());
  println!("加密后bytes===>{:?}", content.to_bytes());
  for byte in &content {
    println!("重新加密:{:?}", byte);
  }
  let mut result: ByteList = vec![];
  encode_to(
    &bytes,
    &EncodeType::NamedOrHex,
    &CharacterSet::SpecialCharsAndNoASCII,
    &mut result,
  );
  println!("result:{:?}", result);
  Ok(())
}
