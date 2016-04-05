extern crate tatar;
use tatar::Tatar;

fn main() {
    Tatar::create_multi_tar("mutliple.tar", vec!["Cargo.toml",".gitignore"]);
}
