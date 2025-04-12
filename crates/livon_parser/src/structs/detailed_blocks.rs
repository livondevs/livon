use super::detailed_language_blocks::DetailedLanguageBlocks;
use super::detailed_meta_data::DetailedMetaData;

#[derive(Debug)]
pub struct DetailedBlock {
    pub detailed_meta_data: Vec<DetailedMetaData>,
    pub detailed_language_blocks: DetailedLanguageBlocks,
}
