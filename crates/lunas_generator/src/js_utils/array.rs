use itertools::Itertools;

pub fn vec_str_to_array(vec: Vec<String>) -> String {
    format!("[{}]", vec.iter().join(", "))
}
