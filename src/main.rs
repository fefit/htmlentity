use htmlentity::entity::{decode, decode_chars, encode};
fn main() {
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
	println!("result:{:?}", result);
	let decoded = decode(&result.iter().collect::<String>());
	println!("decoded:{:?}", decoded);
}
