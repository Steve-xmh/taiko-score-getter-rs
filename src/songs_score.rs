use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Response {
    pub status: i64,
    pub message: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub data: Data,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub userid: String,
    #[serde(rename = "scoreInfo")]
    pub score_info: Vec<ScoreInfo>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoreInfo {
    pub song_no: i64,
    pub level: i64,
    pub high_score: i64,
    pub best_score_rank: i64,
    pub good_cnt: i64,
    pub ok_cnt: i64,
    pub ng_cnt: i64,
    pub pound_cnt: i64,
    pub combo_cnt: i64,
    pub option_flg: Vec<serde_json::Value>,
    pub tone_flg: Vec<i64>,
    pub stage_cnt: i64,
    pub clear_cnt: i64,
    pub full_combo_cnt: i64,
    pub dondaful_combo_cnt: i64,
    pub highscore_datetime: String,
    pub highscore_mode: i64,
    pub update_datetime: String,
    pub song_detail: SongDetail,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SongDetail {
    pub sort: i64,
    pub id: i64,
    pub open_day: String,
    pub type_: String,
    pub song_name_jp: String,
    pub song_name: String,
    pub family: String,
    // pub level_1: u8,
    // pub level_2: u8,
    // pub level_3: u8,
    // pub level_4: u8,
    // pub level_5: serde_json::Value,
}
