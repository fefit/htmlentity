# htmlentity

html entity encode and decode.

[![Docs](https://docs.rs/htmlentity/badge.svg)](https://docs.rs/htmlentity/badge.svg)
[![Build Status](https://travis-ci.com/fefit/htmlentity.svg?branch=main)](https://travis-ci.com/github/fefit/htmlentity)
[![codecov](https://codecov.io/gh/fefit/htmlentity/branch/main/graph/badge.svg)](https://codecov.io/gh/fefit/htmlentity)

## How to use

```rust
use htmlentity::entity::*;

let html = "<div class='header'></div>";
let html_encoded: Vec<char> = encode(html, Entities::SpecialChars, EncodeType::Named);
assert_eq!(html_encoded.iter().collect::<String>(), "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");

let html_decoded: Vec<char> = decode_chars(&html_encoded);
assert_eq!(html, html_decoded.iter().collect::<String>());
```

For more details, please see the document in [Docs.rs](https://docs.rs/htmlentity)

## License

[MIT License](./LICENSE).
