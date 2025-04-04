use crate::js_utils::array::vec_str_to_array;

pub fn gen_binary_map_from_bool(bools: Vec<bool>) -> String {
    if bools.len() <= 31 {
        let mut result = 0u128;
        for (i, &value) in bools.iter().enumerate() {
            if value {
                result |= 2u128.pow(i as u32);
            }
        }
        result.to_string()
    } else {
        let mut result = vec![];
        for chunk in bools.chunks(31) {
            let mut chunk_result = 0u128;
            for (i, &value) in chunk.iter().enumerate() {
                if value {
                    chunk_result |= 2u128.pow(i as u32);
                }
            }
            result.push(chunk_result.to_string());
        }
        vec_str_to_array(result)
    }
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
