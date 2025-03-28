mod for_parser;
mod parse2;
mod parser1;
mod parsers;
pub mod structs;
mod swc_parser;
mod ts_to_js;

use parse2::parse2;
use parser1::parse1;
pub use structs::detailed_blocks::DetailedBlock;
pub use structs::detailed_meta_data::{DetailedMetaData, PropsInput, UseComponentStatement};

pub fn parse_lunas_file(input: &str) -> Result<DetailedBlock, String> {
    let new_input = format!("{}\n", input);
    let parsed_items = match parse1(&new_input) {
        Ok(r) => {
            let (_, parsed_items) = r;
            Ok(parsed_items)
        }
        Err(e) => Err(e.to_string()),
    }?;

    let detailed_block = match parse2(parsed_items) {
        Ok(r) => r,
        Err(e) => return Err(format!("{:?}", e)),
    };

    Ok(detailed_block)
}

pub use for_parser::for_parser::{parse_for_statement, ParsedFor};
