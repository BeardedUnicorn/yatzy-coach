#![recursion_limit = "256"]

mod models;
mod scoring;
mod solver;

use std::collections::HashSet;

use models::{RerollSuggestion, SolveRackRequest, SolveRackResponse, WordRecommendation};
use tauri::Manager;

const DEFAULT_LIMIT: usize = 40;
const REROLL_SUGGESTION_LIMIT: usize = 6;

#[tauri::command]
fn solve_rack_command(request: SolveRackRequest) -> Result<SolveRackResponse, String> {
    let SolveRackRequest {
        rack_letters,
        target_word_length,
        invalid_words,
        rack_bonuses,
        round,
    } = request;

    let normalized_letters: Vec<char> = rack_letters
        .into_iter()
        .flat_map(|entry| entry.chars().next())
        .filter(|ch| ch.is_ascii_alphabetic())
        .map(|ch| ch.to_ascii_uppercase())
        .collect();

    if normalized_letters.is_empty() {
        return Err("Add at least one rack letter before solving.".into());
    }

    if let Some(len) = target_word_length {
        if len < 2 || len > 15 {
            return Err("Target word length must be between 2 and 15.".into());
        }
    }

    let normalized_invalid: HashSet<String> = invalid_words
        .into_iter()
        .map(|word| word.trim().to_ascii_uppercase())
        .filter(|word| !word.is_empty())
        .collect();

    let target_filter = target_word_length.map(|len| len as usize);

    let round_value = round.unwrap_or(1);
    if !(1..=5).contains(&round_value) {
        return Err("Round must be between 1 and 5.".into());
    }
    let round_multiplier = u32::from(round_value);

    let normalized_bonuses: Vec<solver::Bonus> = rack_bonuses
        .into_iter()
        .map(|value| solver::Bonus::from_str_raw(&value))
        .collect();

    let candidates = solver::solve_rack(
        &normalized_letters,
        target_filter,
        &normalized_invalid,
        DEFAULT_LIMIT,
        &normalized_bonuses,
        round_multiplier,
    );

    let recommendations: Vec<WordRecommendation> = candidates
        .into_iter()
        .map(|candidate| WordRecommendation {
            slot_index: None,
            word: candidate.word.clone(),
            score: Some(candidate.score as f64),
            computed_score: Some(candidate.score as f64),
            confidence: None,
            letters_used: candidate.word.chars().map(|ch| ch.to_string()).collect(),
            placement_notes: None,
        })
        .collect();

    let reroll_target = target_filter.unwrap_or(normalized_letters.len());
    let best_word = recommendations.first().map(|rec| rec.word.as_str());
    let reroll_suggestions: Vec<RerollSuggestion> = solver::suggest_rerolls(
        &normalized_letters,
        reroll_target,
        &normalized_invalid,
        REROLL_SUGGESTION_LIMIT,
        best_word,
    )
    .into_iter()
    .map(|advice| RerollSuggestion {
        target_word: advice.target_word,
        missing_letters: advice
            .missing_letters
            .into_iter()
            .map(|ch| ch.to_string())
            .collect(),
        reroll_letters: advice
            .reroll_letters
            .into_iter()
            .map(|ch| ch.to_string())
            .collect(),
        keep_letters: advice
            .keep_letters
            .into_iter()
            .map(|ch| ch.to_string())
            .collect(),
        estimated_score: advice.estimated_score.map(|value| f64::from(value)),
        success_probability: advice.success_probability,
        phase: Some(advice.phase.to_string()),
        notes: advice.notes,
        focus_tags: advice.focus_tags,
    })
    .collect();

    let rack_for_response = normalized_letters
        .into_iter()
        .map(|ch| ch.to_string())
        .collect();
    let bonuses_for_response = normalized_bonuses
        .iter()
        .map(|bonus| bonus.as_code().to_string())
        .collect();

    Ok(SolveRackResponse {
        rack_letters: rack_for_response,
        target_word_length,
        rack_bonuses: bonuses_for_response,
        round: Some(round_value),
        recommendations,
        reroll_suggestions,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![solve_rack_command])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.maximize();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
