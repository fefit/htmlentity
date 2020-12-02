use htmlentity::entity::{
  decode, encode, encode_default, encode_filter, encode_with, EncodeType, EntitySet, NOOP,
};

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
  assert_eq!(
    encode(content, EntitySet::SpecialChars, Default::default()),
    encoded_content
  );
  assert_eq!(decode(encoded_content), content);
  // test 3
  let content = "<div>";
  let encoded_content = "&lt;&#x64;&#x69;&#x76;&gt;";
  assert_eq!(
    encode(content, EntitySet::All, EncodeType::NamedOrHex),
    encoded_content
  );
  assert_eq!(decode(encoded_content), content);
  // test 4
  let content = "\t<div>";
  let encoded_content = encode_filter(
    content,
    |_ch: char| true,
    EncodeType::Named,
    Some(|ch| ch == '<'),
  );
  assert_eq!(encoded_content, "&Tab;<div&gt;");
  // test 5
  let content = "\t<div>";
  let encoded_content = encode_with(content, |ch: char| {
    if ch == '<' {
      return None;
    }
    Some(EncodeType::Named)
  });
  assert_eq!(encoded_content, "&Tab;<div&gt;");
}

#[test]
fn test_wrong_entity() {
  let content = "&#;";
  assert_eq!(decode(content), content);
  let content = "&;";
  assert_eq!(decode(content), content);
}
#[test]
fn test_decode_named() {
  // wrong named
  let content = "&#q123;";
  assert_eq!(decode(content), content);
  let content = "&123;";
  assert_eq!(decode(content), content);
  let content = "&q123;";
  assert_eq!(decode(content), content);
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
  // wrong hex or decimal
  let content = "&#a00;";
  assert_eq!(decode(content), content);
}

#[test]
fn test_decode_decimal() {
  // decimal
  let content = "&#8594;";
  assert_eq!(decode(content), "→");
  // decimal with leading zeros
  let content = "&#0008594;";
  assert_eq!(decode(content), "→");
  // wrong decimal unicode ranges
  let content = "&#1114112;";
  assert_eq!(decode(content), content);
  let content = "&#56320;";
  assert_eq!(decode(content), content);
}

#[test]
fn test_exclude_named() {
  let html = "<div class='header'></div>";
  let html_encoded = encode_filter(
    html,
    |ch| ch != '<' && EntitySet::SpecialChars.contains(&ch),
    EncodeType::Named,
    NOOP,
  );
  assert_eq!(html_encoded, "<div class=&apos;header&apos;&gt;</div&gt;");

  // special characters, but exclude the single quote "'" use named.
  let html = "<div class='header'></div>";
  let html_encoded = encode_filter(
    html,
    |ch| EntitySet::SpecialChars.contains(&ch),
    EncodeType::NamedOrDecimal,
    Some(|ch| ch == '\''),
  );
  assert_eq!(
    html_encoded,
    "&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;"
  );

  // the same as the EncodeType::Decimal.
  let html = "<div class='header'></div>";
  let html_encoded = encode_filter(
    html,
    |ch| EntitySet::SpecialChars.contains(&ch),
    EncodeType::NamedOrDecimal,
    Some(|ch| EntitySet::SpecialChars.contains(&ch)),
  );
  assert_eq!(
    html_encoded,
    "&#60;div class=&#39;header&#39;&#62;&#60;/div&#62;"
  );
}
