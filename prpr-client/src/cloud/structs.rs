use super::LCObject;
use crate::data::BriefChartInfo;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct LCFile {
    pub url: String,
}

#[derive(Clone, Deserialize)]
pub struct Pointer {
    #[serde(rename = "objectId")]
    pub id: String,
}

impl From<String> for Pointer {
    fn from(id: String) -> Self {
        Self { id }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "objectId")]
    pub id: String,
    #[serde(rename = "username")]
    pub name: String,
    pub short_id: String,
    pub email: String,
}

#[derive(Clone, Deserialize)]
pub struct ChartItemData {
    #[serde(rename = "objectId")]
    pub id: String,

    pub uploader: Pointer,

    #[serde(flatten)]
    pub info: BriefChartInfo,

    pub file: LCFile,
    pub illustration: LCFile,
}

impl LCObject for ChartItemData {
    const CLASS_NAME: &'static str = "Chart";
}