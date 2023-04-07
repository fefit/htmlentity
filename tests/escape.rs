use std::borrow::Cow;

use htmlentity::{
  entity::{
    decode, decode_chars, decode_chars_to, decode_to, encode, encode_char, encode_with,
    CharacterSet, EncodeType, EntityType, ICodedDataTrait,
  },
  types::AnyhowResult,
};

fn decode_to_string(content: &str) -> String {
  if let Ok(result) = decode(content.as_bytes()).to_string() {
    result
  } else {
    String::from("")
  }
}
#[test]
fn test_escape() -> AnyhowResult<()> {
  // -------------------- test 1 --------------------
  let content = "
    \t
    \n
    <br>this is a multiple line text.
    <div class='product'>
      <span><span>￥</span>100</span>
      <h4>this is a title&lt;main&gt;</h4>
    </div>
  ";
  let result = encode(content.as_bytes(), &Default::default(), &Default::default());
  let encoded_string = result.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(decode_to_string(&encoded_string?), content);
  let encoded_chars = result.to_chars();
  assert!(encoded_chars.is_ok());
  let cur_chars = encoded_chars?;
  assert_eq!(decode_chars(&cur_chars).iter().collect::<String>(), content);
  assert_eq!(decode_chars(&['&', ';']).iter().collect::<String>(), "&;");
  assert_eq!(
    decode_chars(&['&', ';', '&', 'l', 't', ';'])
      .iter()
      .collect::<String>(),
    "&;<"
  );
  // decode chars to
  let mut decode_result: Vec<char> = Vec::new();
  decode_chars_to(&cur_chars, &mut decode_result);
  assert_eq!(decode_result.iter().collect::<String>(), content);
  // decode
  let mut data = vec![];
  decode_to(&result.to_bytes(), &mut data);
  let now_content = std::str::from_utf8(&data);
  assert!(now_content.is_ok());
  assert_eq!(now_content?, content);
  // ----------------- test 2 -------------------
  let content = "<div>&nbsp;'\"</div>";
  let encoded_content = "&lt;div&gt;&amp;nbsp;&apos;&quot;&lt;/div&gt;";
  let encoded_data = encode(
    content.as_bytes(),
    &EncodeType::NamedOrHex,
    &CharacterSet::SpecialChars,
  );
  let encoded_string = encoded_data.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(encoded_string?, encoded_content);
  assert_eq!(decode_to_string(encoded_content), content);
  // ------------------- test 3 -------------------
  let content = "<div>&nbsp;'\"</div>";
  let encoded_content = "&lt;div&gt;&amp;nbsp;'\"&lt;/div&gt;";
  let encoded_data = encode(
    content.as_bytes(),
    &EncodeType::NamedOrHex,
    &CharacterSet::Html,
  );
  let encoded_string = encoded_data.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(encoded_string?, encoded_content);
  assert_eq!(decode_to_string(encoded_content), content);
  // -------------------- test 3 --------------------
  let content = "<div>℗ℑ";
  let encoded_content = "&lt;&#x64;&#x69;&#x76;&gt;&copysr;&image;";
  let encoded_data = encode(
    content.as_bytes(),
    &EncodeType::NamedOrHex,
    &CharacterSet::All,
  );
  let encoded_string = encoded_data.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(encoded_string?, encoded_content);
  assert_eq!(decode_to_string(encoded_content), content);
  // -------------------- test 4 --------------------
  let content = "\t<div>";
  let encoded_data = encode_with(content.as_bytes(), &EncodeType::Named, |ch, _| {
    if *ch == '<' {
      return (false, None);
    }
    (true, None)
  });
  let encoded_string = encoded_data.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(encoded_string?, "&Tab;<div&gt;");
  Ok(())
}

#[test]
fn test_wrong_entity() {
  let content = "&#;";
  assert_eq!(decode_to_string(content), content);
  let content = "&;";
  assert_eq!(decode_to_string(content), content);
}
#[test]
fn test_decode_named() {
  // wrong named
  let content = "&#q123;";
  let mut decoded_data = decode(content.as_bytes());
  assert!(!decoded_data.is_ok());
  assert!(!decoded_data.get_errors().is_empty());
  assert_eq!(decoded_data.entity_count(), 0);
  decoded_data.to_owned();
  assert_eq!(decoded_data.into_bytes(), content.as_bytes());
  assert_eq!(decode_to_string(content), content);
  let content = "&123;";
  assert_eq!(decode_to_string(content), content);
  let content = "&q123;";
  assert_eq!(decode_to_string(content), content);
}

#[test]
fn test_decode_hex() {
  // hex
  let content = "&#x2192;";
  assert_eq!(decode_to_string(content), "→");
  // HEX
  let content = "&#X2192;";
  assert_eq!(decode_to_string(content), "→");
  // hex with leading zeros
  let content = "&#x0002192;";
  assert_eq!(decode_to_string(content), "→");
  // wrong hex unicode ranges
  let content = "&#x110000;";
  assert_eq!(decode_to_string(content), content);
  let content = "&#xDC00;";
  assert_eq!(decode_to_string(content), content);
  // wrong hex
  let content = "&#xa0fh;";
  assert_eq!(decode_to_string(content), content);
  // wrong hex or decimal
  let content = "&#a00;";
  assert_eq!(decode_to_string(content), content);
}

#[test]
fn test_decode_decimal() {
  // decimal
  let content = "&#8594;";
  assert_eq!(decode_to_string(content), "→");
  // decimal with leading zeros
  let content = "&#0008594;";
  assert_eq!(decode_to_string(content), "→");
  // wrong decimal unicode ranges
  let content = "&#1114112;";
  assert_eq!(decode_to_string(content), content);
  let content = "&#56320;";
  assert_eq!(decode_to_string(content), content);
}

#[test]
fn test_exclude_named() -> AnyhowResult<()> {
  let html = "<div class='header'>℗</div>";
  let encode_type = EncodeType::Named;
  let entity_set = CharacterSet::SpecialCharsAndNonASCII;
  let html_encoded = encode_with(html.as_bytes(), &encode_type, |ch, _| {
    if *ch == '<' {
      return (false, None);
    }
    return entity_set.filter(ch, &encode_type);
  });
  let encoded_string = html_encoded.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(
    encoded_string?,
    String::from("<div class=&apos;header&apos;&gt;&copysr;</div&gt;")
  );

  // special characters, but exclude the single quote "'" use named.
  let html = "<div class='header'></div>";
  let html_encoded = encode_with(
    html.as_bytes(),
    &EncodeType::NamedOrDecimal,
    |ch, encode_type| {
      if *ch == '\'' {
        return (
          true,
          Some((EntityType::Decimal, Cow::Owned(b"39".to_vec()))),
        );
      }
      return entity_set.filter(ch, encode_type);
    },
  );
  let encoded_string = html_encoded.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(
    encoded_string?,
    "&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;"
  );

  // the same as the EncodeType::Decimal.
  let html = "<div class='header'></div>";
  let html_encoded = encode_with(
    html.as_bytes(),
    &EncodeType::NamedOrDecimal,
    |ch, encode_type| {
      let (need_encode, _) = entity_set.filter(ch, encode_type);
      if need_encode {
        if let Some(char_entity) = encode_char(ch, &EncodeType::Decimal) {
          return (
            true,
            Some((EntityType::Decimal, Cow::from(char_entity.data()))),
          );
        }
      }
      (false, None)
    },
  );
  let encoded_string = html_encoded.to_string();
  assert!(encoded_string.is_ok());
  assert_eq!(
    encoded_string?,
    "&#60;div class=&#39;header&#39;&#62;&#60;/div&#62;"
  );
  Ok(())
}

#[test]
fn test_unexpected() -> AnyhowResult<()> {
  assert_eq!(decode(b"&").to_string()?, "&");
  assert_eq!(decode(b"&;").to_string()?, "&;");
  assert_eq!(decode(b"&a0;").to_string()?, "&a0;");
  assert_eq!(decode(b"&0a;").to_string()?, "&0a;");
  assert_eq!(decode(b"&#").to_string()?, "&#");
  assert_eq!(decode(b"&#;").to_string()?, "&#;");
  assert_eq!(decode(b"&#a;").to_string()?, "&#a;");
  assert_eq!(decode(b"&#x;").to_string()?, "&#x;");
  assert_eq!(decode(b"&#xg;").to_string()?, "&#xg;");
  assert_eq!(decode(b"&#x0g;").to_string()?, "&#x0g;");
  assert_eq!(decode(b"abc&").to_string()?, "abc&");
  assert_eq!(decode(b"&#1&lt;").to_string()?, "&#1<");
  Ok(())
}
