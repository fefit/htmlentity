use htmlentity::entity::decode;
fn main(){
  println!("{}", decode("&#q123;"))
}