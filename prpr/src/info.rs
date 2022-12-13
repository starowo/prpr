use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartFormat {
    Rpe,
    Pec,
    Pgr,
}

#[derive(Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct ChartInfo {
    pub id: Option<String>,

    pub name: String,
    pub level: String,
    pub charter: String,
    pub composer: String,
    pub illustrator: String,

    pub chart: String,
    pub format: Option<ChartFormat>,
    pub music: String,
    pub illustration: String,

    pub aspect_ratio: f32,

    pub intro: String,
    pub tags: Vec<String>,
}

impl Default for ChartInfo {
    fn default() -> Self {
        Self {
            id: None,

            name: "UK".to_string(),
            level: "UK Lv.?".to_string(),
            charter: "UK".to_string(),
            composer: "UK".to_string(),
            illustrator: "UK".to_string(),

            chart: "chart.json".to_string(),
            format: None,
            music: "song.mp3".to_string(),
            illustration: "background.png".to_string(),

            aspect_ratio: 16. / 9.,

            intro: String::new(),
            tags: Vec::new(),
        }
    }
}