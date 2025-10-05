use key::register_any_key;

pub fn main() {
    register_any_key(|key| {
        println!("{key} is pressed");
    });
    // 等待输入
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "exit" {
            break;
        }
    }
}
