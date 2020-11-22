# htmlentity
html entity encode and decode.

[![Build Status](https://travis-ci.com/fefit/htmlentity.svg?branch=main)](https://travis-ci.com/github/fefit/htmlentity)
[![codecov](https://codecov.io/gh/fefit/htmlentity/branch/main/graph/badge.svg)](https://codecov.io/gh/fefit/htmlentity)

## How to use

```rust
use htmlentity::entity::*;

let html = "<div class='header'></div>";
let html_encoded = encode(html, Entities::SpecialChars, EncodeType::Named);
assert_eq!(html_encoded, "&lt;div class=&apos;header&apos;&gt;&lt;/div&gt;");

let html_decoded = decode(&html_encoded);
assert_eq!(html, html_decoded);
```
For more details, please see the document in [crates.io](https://crates.io) 