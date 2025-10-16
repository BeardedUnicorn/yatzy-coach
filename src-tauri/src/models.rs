use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct WordRecommendation {
    #[serde(default)]
    pub slot_index: Option<u8>,
    pub word: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub computed_score: Option<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub letters_used: Vec<String>,
    #[serde(default)]
    pub placement_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SolveRackRequest {
    pub rack_letters: Vec<String>,
    #[serde(default)]
    pub target_word_length: Option<u8>,
    #[serde(default)]
    pub invalid_words: Vec<String>,
    #[serde(default)]
    pub rack_bonuses: Vec<String>,
    #[serde(default)]
    pub round: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SolveRackResponse {
    pub rack_letters: Vec<String>,
    #[serde(default)]
    pub target_word_length: Option<u8>,
    #[serde(default)]
    pub rack_bonuses: Vec<String>,
    #[serde(default)]
    pub round: Option<u8>,
    #[serde(default)]
    pub recommendations: Vec<WordRecommendation>,
    #[serde(default)]
    pub reroll_suggestions: Vec<RerollSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct RerollSuggestion {
    pub target_word: String,
    #[serde(default)]
    pub missing_letters: Vec<String>,
    #[serde(default)]
    pub reroll_letters: Vec<String>,
    #[serde(default)]
    pub keep_letters: Vec<String>,
    #[serde(default)]
    pub estimated_score: Option<f64>,
    #[serde(default)]
    pub success_probability: Option<f64>,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}
