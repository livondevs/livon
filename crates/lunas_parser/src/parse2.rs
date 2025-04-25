use std::collections::HashMap;

use crate::structs::blocks::{LanguageBlock, ParsedItem};

use crate::structs::detailed_blocks::DetailedBlock;
use crate::structs::detailed_language_blocks::{DetailedLanguageBlocks, JsBlock};
use crate::structs::detailed_meta_data::DetailedMetaData;
use crate::swc_parser::parse_with_swc;
use crate::ts_to_js::transform_ts_to_js;

use lunas_html_parser::Dom;

pub fn parse2(input: Vec<ParsedItem>) -> Result<DetailedBlock, String> {
    let variant_a_values: Vec<LanguageBlock> = input
        .clone()
        .into_iter()
        .filter_map(|e| match e {
            ParsedItem::LanguageBlock(bl) => Some(bl),
            _ => None,
        })
        .collect();
    let lang_blocks = parse_language_blocks(variant_a_values)?;

    let detailed_meta_data = input
        .into_iter()
        .filter_map(|e| match e {
            ParsedItem::MetaData(meta) => Some(meta),
            _ => None,
        })
        .map(|e| DetailedMetaData::from_simple_meta_data(e))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DetailedBlock {
        detailed_meta_data: detailed_meta_data,
        detailed_language_blocks: lang_blocks,
    })
}

fn parse_language_blocks<'a>(blks: Vec<LanguageBlock>) -> Result<DetailedLanguageBlocks, String> {
    let mut hm = HashMap::new();
    for block in &blks {
        let language_name: &str = &block.language_name.as_str();
        // if language_name is not one of 'html', 'style', 'script'
        if language_name != "html" && language_name != "style" && language_name != "script" {
            return Err("Invalid language name".to_string());
        }
        if hm.contains_key(language_name) {
            return Err("Duplicate language name".to_string());
        }
        let content = block.content.clone();

        hm.insert(language_name, content);
    }

    let html = hm.get("html");
    if html == None {
        return Err("Missing html block".to_string());
    }
    let parsed_html_dom_result = Dom::parse(html.unwrap());
    match parsed_html_dom_result {
        Ok(parsed_html) => {
            let css = hm.get("style");
            let ts = hm.get("script");
            let parsed_js = match ts {
                Some(ts) => {
                    let js = transform_ts_to_js(ts).map_err(|e| e.to_string())?;
                    let parsed = parse_with_swc(&js);
                    let parsed_json = serde_json::to_value(&parsed).unwrap();
                    Some(JsBlock {
                        ast: parsed_json,
                        raw: js.trim().into(),
                    })
                }
                None => None,
            };
            let str_css = match css {
                Some(css) => Some(css.to_string()),
                None => None,
            };
            Ok(DetailedLanguageBlocks {
                dom: parsed_html,
                css: str_css,
                js: parsed_js,
            })
        }
        Err(_) => return Err("Invalid html block".to_string()),
    }
}
