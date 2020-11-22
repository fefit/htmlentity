use htmlentity::entity::{decode, encode, encode_default, Entities };

#[test]
fn test_escape() {
  // test 1
  let content = "
    \t
    \n
    <br>this is a multiple line text.
    <div class='product'>
      <span><span>￥</span>100</span>
      <h4>this is a title&lt;main&gt;</h4>
    </div>
  ";
  let result = encode_default(content);
  assert_eq!(decode(&result), content);
  // test 2
  let content = "<div>";
  let encoded_content = "&lt;div&gt;";
  assert_eq!(encode(content, Entities::SpecialChars, Default::default()), encoded_content);
  assert_eq!(decode(encoded_content), content);
}

#[test]
fn test_decode_hex() {
  // hex
  let content = "&#x2192;";
  assert_eq!(decode(content), "→");
  // HEX
  let content = "&#X2192;";
  assert_eq!(decode(content), "→");
  // hex with leading zeros
  let content = "&#x0002192;";
  assert_eq!(decode(content), "→");
  // wrong hex unicode ranges
  let content = "&#x110000;";
  assert_eq!(decode(content), content);
  let content = "&#xDC00;";
  assert_eq!(decode(content), content);
  // wrong hex
  let content = "&#xa0fh;";
  assert_eq!(decode(content), content);
}

#[test]
fn test_decode_number() {
  // number
  let content = "&#8594;";
  assert_eq!(decode(content), "→");
  // number with leading zeros
  let content = "&#0008594;";
  assert_eq!(decode(content), "→");
  // wrong number unicode ranges
  let content = "&#1114112;";
  assert_eq!(decode(content), content);
  let content = "&#56320;";
  assert_eq!(decode(content), content);
}
