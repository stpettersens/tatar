extern crate tatar;
use tatar::Tatar;

fn main() {
    Tatar::create_single_tar("single.tar", "Cargo.toml");
}
