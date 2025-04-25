pub fn generate_import_string(imports: &Vec<String>) -> String {
    match imports.len() == 0 {
        true => String::new(),
        false => imports
            .iter()
            .map(|import| format!("\n{}", import))
            .collect::<String>(),
    }
}
