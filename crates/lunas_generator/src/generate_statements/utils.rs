pub fn gen_binary_map_from_bool(bools: Vec<bool>) -> u32 {
    let mut result = 0;
    for (i, &value) in bools.iter().enumerate() {
        if value {
            result |= 2u32.pow(i as u32);
        }
    }
    result
}

// TODO: インデントの種類を入力によって変えられるようにする
pub fn create_indent(string: &str) -> String {
    let mut output = "".to_string();
    let indent = "    ";
    for (i, line) in string.lines().into_iter().enumerate() {
        match line == "" {
            true => {}
            false => {
                output.push_str(indent);
                output.push_str(line);
            }
        }
        if i != string.lines().into_iter().count() - 1 {
            output.push_str("\n");
        }
    }
    output
}
