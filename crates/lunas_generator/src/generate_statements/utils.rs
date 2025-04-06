use num_bigint::BigUint;

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

/// Returns a binary number that is the result of ORing all the numbers in the argument.
/// ```
/// let numbers = vec![0b0001, 0b0010, 0b0100];
/// let result = get_combined_binary_number(numbers);
/// assert_eq!(result, 0b0111);
/// ```
pub fn get_combined_binary_number(numbers: Vec<BigUint>) -> String {
    let mut result = BigUint::ZERO;
    for value in &numbers {
        result |= value.clone();
    }
    let num_arr = bit_num_to_array(result)
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    if num_arr.len() == 1 {
        return num_arr[0].clone();
    }
    vec_str_to_array(
        num_arr
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
}

use num_traits::Zero;

fn bit_num_to_array(number: BigUint) -> Vec<u32> {
    let bin_str = number.to_str_radix(2);
    let len = bin_str.len();
    let mut chunks = Vec::new();

    if number.is_zero() {
        return vec![0];
    }

    let mut start = 0;
    while start < len {
        let end = if start + 31 <= len { start + 31 } else { len };

        let chunk_str = &bin_str[len - end..len - start];
        let chunk =
            u32::from_str_radix(chunk_str, 2).expect("Failed to parse binary string to u32");
        chunks.push(chunk);
        start = end;
    }
    chunks
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use num_traits::Num;

    macro_rules! generate_tests {
        ($($name:ident: $input:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let num = BigUint::from_str_radix($input, 2).unwrap();
                    let result = bit_num_to_array(num);
                    assert_eq!(result, $expected, "Failed for input: {}", $input);
                }
            )*
        };
    }

    generate_tests! {
        test_case1: "1000000000000000000000000000000111" => vec![7,4],
        test_case2: "111" => vec![7],
        test_case3: "1000000000000000000000000000000000000000000000000000000000000001" => vec![1,0,2],
        test_case4: "1111111111111111111111111111111111" => vec![2147483647,7],
    }
}
