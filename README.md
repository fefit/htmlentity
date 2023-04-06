# htmlentity

A library for encoding and decoding HTML entities.

[![Docs](https://docs.rs/htmlentity/badge.svg)](https://docs.rs/htmlentity)
[![Build Status](https://github.com/fefit/htmlentity/actions/workflows/test.yml/badge.svg)](https://github.com/fefit/htmlentity/actions)
[![codecov](https://codecov.io/gh/fefit/htmlentity/branch/main/graph/badge.svg)](https://codecov.io/gh/fefit/htmlentity)

## How to use

```rust
use htmlentity::entity::{ encode, decode, EncodeType, CharacterSet, ICodedDataTrait };
use htmlentity::types::{ AnyhowResult, Byte };
fn main() -> AnyhowResult<()> {
  let html = "<div name='htmlentity'>Hello!世界!</div>";
  let html_after_encoded = "&lt;div name='htmlentity'&gt;Hello!&#x4e16;&#x754c;!&lt;/div&gt;";
  // encode
  let encoded_data = encode(html.as_bytes(), &EncodeType::NamedOrHex, &CharacterSet::HtmlAndNonASCII);
  assert_eq!(encoded_data.to_bytes(), html_after_encoded.as_bytes());
  assert_eq!(encoded_data.to_string()?, String::from(html_after_encoded));
  assert_eq!(encoded_data.to_chars()?, String::from(html_after_encoded).chars().collect::<Vec<char>>());
  // decode
  let bytes = encoded_data.into_iter().map(|(byte, _)| *byte).collect::<Vec<Byte>>();
  let decoded_data = decode(&bytes);
  assert_eq!(decoded_data.to_bytes(), html.as_bytes());
  assert_eq!(decoded_data.to_string()?, String::from(html));
  assert_eq!(decoded_data.to_chars()?, String::from(html).chars().collect::<Vec<char>>());
  Ok(())
}
```

For more details, please see the document in [Docs.rs](https://docs.rs/htmlentity)

## License

[MIT License](./LICENSE).
