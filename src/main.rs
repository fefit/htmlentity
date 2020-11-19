use htmlentity::entity::encode;
fn main() {
    let ch = "\t
        abhaha&agsdgs&\"this is a test
    ";
    let result = encode(ch);
    println!("result is {:?}", result);
}
