use htmlentity::entity::{
	decode, decode_chars, decode_chars_to, decode_to, encode, encode_filter, encode_with, EncodeType,
	EntitySet, NOOP,
};

fn decode_to_string(content: &str) -> String {
	decode(content).iter().collect::<String>()
}
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
	let result = encode(content, Default::default(), Default::default());
	assert_eq!(
		decode_to_string(&result.iter().collect::<String>()),
		content
	);
	assert_eq!(decode_chars(&result).iter().collect::<String>(), content);
	// decode chars to
	let mut decode_result: Vec<char> = Vec::new();
	decode_chars_to(&result, &mut decode_result);
	assert_eq!(decode_result.iter().collect::<String>(), content);
	// decode
	let now_content = result.iter().collect::<String>();
	let mut result = String::new();
	decode_to(&now_content, &mut result);
	assert_eq!(result, content);
	// test 2
	let content = "<div>&nbsp;'\"</div>";
	let encoded_content = "&lt;div&gt;&amp;nbsp;&apos;&quot;&lt;/div&gt;";
	assert_eq!(
		encode(content, EntitySet::SpecialChars, Default::default())
			.iter()
			.collect::<String>(),
		encoded_content
	);
	assert_eq!(decode_to_string(encoded_content), content);
	// test 3
	let content = "<div>&nbsp;'\"</div>";
	let encoded_content = "&lt;div&gt;&amp;nbsp;'\"&lt;/div&gt;";
	assert_eq!(
		encode(content, EntitySet::Html, Default::default())
			.iter()
			.collect::<String>(),
		encoded_content
	);
	assert_eq!(decode_to_string(encoded_content), content);
	// test 3
	let content = "<div>";
	let encoded_content = "&lt;&#x64;&#x69;&#x76;&gt;";
	assert_eq!(
		encode(content, EntitySet::All, EncodeType::NamedOrHex)
			.iter()
			.collect::<String>(),
		encoded_content
	);
	assert_eq!(decode_to_string(encoded_content), content);
	// test 4
	let content = "\t<div>";
	let encoded_content = encode_filter(
		&content.chars().collect::<Vec<char>>(),
		|_ch: &char| true,
		EncodeType::Named,
		Some(|ch: &char| *ch == '<'),
	)
	.iter()
	.collect::<String>();
	assert_eq!(encoded_content, "&Tab;<div&gt;");
	// test 5
	let content = "\t<div>";
	let encoded_content = encode_with(&content.chars().collect::<Vec<char>>(), |ch: &char| {
		if *ch == '<' {
			return None;
		}
		Some(EncodeType::Named)
	})
	.iter()
	.collect::<String>();
	assert_eq!(encoded_content, "&Tab;<div&gt;");
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
fn test_exclude_named() {
	let html = "<div class='header'></div>";
	let html_encoded = encode_filter(
		&html.chars().collect::<Vec<char>>(),
		|ch: &char| *ch != '<' && EntitySet::SpecialChars.contains(ch),
		EncodeType::Named,
		NOOP,
	)
	.iter()
	.collect::<String>();
	assert_eq!(html_encoded, "<div class=&apos;header&apos;&gt;</div&gt;");

	// special characters, but exclude the single quote "'" use named.
	let html = "<div class='header'></div>";
	let html_encoded = encode_filter(
		&html.chars().collect::<Vec<char>>(),
		|ch| EntitySet::SpecialChars.contains(&ch),
		EncodeType::NamedOrDecimal,
		Some(|ch: &char| *ch == '\''),
	)
	.iter()
	.collect::<String>();
	assert_eq!(
		html_encoded,
		"&lt;div class=&#39;header&#39;&gt;&lt;/div&gt;"
	);

	// the same as the EncodeType::Decimal.
	let html = "<div class='header'></div>";
	let html_encoded = encode_filter(
		&html.chars().collect::<Vec<char>>(),
		|ch: &char| EntitySet::SpecialChars.contains(ch),
		EncodeType::NamedOrDecimal,
		Some(|ch: &char| EntitySet::SpecialChars.contains(ch)),
	)
	.iter()
	.collect::<String>();
	assert_eq!(
		html_encoded,
		"&#60;div class=&#39;header&#39;&#62;&#60;/div&#62;"
	);
}

#[test]
fn test_unexpected() {
	assert_eq!(decode("&").iter().collect::<String>(), "&");
	assert_eq!(decode("&;").iter().collect::<String>(), "&;");
	assert_eq!(decode("&a0;").iter().collect::<String>(), "&a0;");
	assert_eq!(decode("&0a;").iter().collect::<String>(), "&0a;");
	assert_eq!(decode("&#").iter().collect::<String>(), "&#");
	assert_eq!(decode("&#;").iter().collect::<String>(), "&#;");
	assert_eq!(decode("&#a;").iter().collect::<String>(), "&#a;");
	assert_eq!(decode("&#x;").iter().collect::<String>(), "&#x;");
	assert_eq!(decode("&#xg;").iter().collect::<String>(), "&#xg;");
	assert_eq!(decode("&#x0g;").iter().collect::<String>(), "&#x0g;");
	assert_eq!(decode("abc&").iter().collect::<String>(), "abc&");
}
