use std::collections::HashSet;

use once_cell::sync::Lazy;

use crate::scoring;

static CSPELL_WORDS: Lazy<HashSet<String>> = Lazy::new(load_cspell_words);
static WORD_BLOCKLIST: Lazy<HashSet<String>> = Lazy::new(load_blocklist);
static CORE_WORDS: Lazy<HashSet<String>> = Lazy::new(load_core_words);
static DICTIONARY: Lazy<Vec<String>> = Lazy::new(load_dictionary);

const LETTER_BAG_COUNTS: [u8; 26] = [
    9, 2, 2, 4, 12, 2, 3, 2, 9, 1, 1, 4, 2, 6, 8, 2, 1, 6, 4, 6, 4, 2, 2, 1, 2, 1,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bonus {
    None,
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

impl Bonus {
    pub fn from_str_raw(value: &str) -> Self {
        match value.trim().to_ascii_uppercase().as_str() {
            "DL" => Bonus::DoubleLetter,
            "TL" => Bonus::TripleLetter,
            "DW" => Bonus::DoubleWord,
            "TW" => Bonus::TripleWord,
            _ => Bonus::None,
        }
    }

    pub fn as_code(self) -> &'static str {
        match self {
            Bonus::None => "NONE",
            Bonus::DoubleLetter => "DL",
            Bonus::TripleLetter => "TL",
            Bonus::DoubleWord => "DW",
            Bonus::TripleWord => "TW",
        }
    }

    fn letter_multiplier(self) -> u32 {
        match self {
            Bonus::DoubleLetter => 2,
            Bonus::TripleLetter => 3,
            _ => 1,
        }
    }

    fn word_multiplier(self) -> u32 {
        match self {
            Bonus::DoubleWord => 2,
            Bonus::TripleWord => 3,
            _ => 1,
        }
    }
}

fn load_dictionary() -> Vec<String> {
    let mut words: Vec<String> = include_str!("data/wordlist.txt")
        .lines()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .filter(|w| CORE_WORDS.contains(&w.to_ascii_lowercase()))
        .map(|w| w.to_ascii_uppercase())
        .filter(|w| w.len() >= 2 && w.len() <= 15 && w.chars().all(|c| c.is_ascii_alphabetic()))
        .filter(|w| CSPELL_WORDS.contains(w))
        .filter(|w| contains_vowel(w))
        .filter(|w| !is_uniform_character(w))
        .filter(|w| !WORD_BLOCKLIST.contains(w))
        .collect();
    words.sort();
    words.dedup();
    words
}

fn load_cspell_words() -> HashSet<String> {
    include_str!("data/cspell-words.txt")
        .lines()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_uppercase())
        .collect()
}

fn load_blocklist() -> HashSet<String> {
    include_str!("data/invalid-words.txt")
        .lines()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .filter(|w| !w.starts_with('#'))
        .map(|w| w.to_ascii_uppercase())
        .collect()
}

fn load_core_words() -> HashSet<String> {
    include_str!("data/core_words.txt")
        .lines()
        .map(|w| w.trim().to_ascii_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

fn contains_vowel(word: &str) -> bool {
    word.chars()
        .any(|ch| matches!(ch, 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'))
}

fn is_uniform_character(word: &str) -> bool {
    let mut chars = word.chars();
    if let Some(first) = chars.next() {
        chars.all(|ch| ch == first)
    } else {
        false
    }
}

#[derive(Debug, Clone)]
pub struct RackCandidate {
    pub word: String,
    pub score: u32,
}

#[derive(Debug, Clone)]
pub struct RerollAdvice {
    pub target_word: String,
    pub missing_letters: Vec<char>,
    pub reroll_letters: Vec<char>,
    pub keep_letters: Vec<char>,
    pub estimated_score: Option<u32>,
    pub success_probability: Option<f64>,
    pub phase: &'static str,
    pub notes: Vec<String>,
    pub focus_tags: Vec<String>,
}

pub fn solve_rack(
    letters: &[char],
    target_length: Option<usize>,
    invalid: &HashSet<String>,
    limit: usize,
    bonuses: &[Bonus],
    round_multiplier: u32,
) -> Vec<RackCandidate> {
    if letters.is_empty() {
        return Vec::new();
    }

    let rack_counts = letter_counts(letters);
    let mut candidates: Vec<RackCandidate> = DICTIONARY
        .iter()
        .filter(|word| target_length.map_or(true, |len| word.len() == len))
        .filter(|word| word.len() <= letters.len())
        .filter(|word| !invalid.contains(*word))
        .filter(|word| word_fits(word, &rack_counts))
        .filter_map(|word| {
            score_word_with_bonuses(word, bonuses, round_multiplier).map(|score| RackCandidate {
                word: word.clone(),
                score,
            })
        })
        .collect();

    candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.word.cmp(&b.word)));

    let max_candidates = limit.max(1);
    if candidates.len() > max_candidates {
        candidates.truncate(max_candidates);
    }

    candidates
}

pub fn suggest_rerolls(
    letters: &[char],
    target_length: usize,
    _invalid: &HashSet<String>,
    limit: usize,
    baseline_word: Option<&str>,
) -> Vec<RerollAdvice> {
    if letters.is_empty() {
        return Vec::new();
    }

    let effective_target = if target_length == 0 {
        letters.len()
    } else {
        target_length
    };

    let baseline_counts = baseline_word.map(|word| letter_counts_str(word));

    let pass_one = analyze_pass_one(letters, effective_target, baseline_counts.as_ref());
    let mut advice = vec![pass_one.to_advice()];

    if advice.len() >= limit {
        advice.truncate(limit);
        return advice;
    }

    if let Some(pass_two) = analyze_pass_two(
        letters,
        effective_target,
        &pass_one,
        baseline_counts.as_ref(),
    ) {
        advice.push(pass_two);
    }

    if advice.len() > limit {
        advice.truncate(limit);
    }

    advice
}

struct PassOneOutcome {
    keep_flags: Vec<bool>,
    keep_letters: Vec<char>,
    reroll_letters: Vec<char>,
    desired_letters: Vec<char>,
    notes: Vec<String>,
    vowel_min: usize,
    focus_tags: Vec<String>,
}

impl PassOneOutcome {
    fn to_advice(&self) -> RerollAdvice {
        let probability = approximate_draw_probability(
            &self.keep_letters,
            &self.reroll_letters,
            &self.desired_letters,
        );
        RerollAdvice {
            target_word: "Pass 1 – Balance rack".to_string(),
            missing_letters: self.desired_letters.clone(),
            reroll_letters: self.reroll_letters.clone(),
            keep_letters: self.keep_letters.clone(),
            estimated_score: None,
            success_probability: probability,
            phase: "foundation",
            notes: self.notes.clone(),
            focus_tags: self.focus_tags.clone(),
        }
    }
}

const GLUE_CONSONANTS: &[char] = &['R', 'S', 'T', 'L', 'N', 'D', 'M', 'P', 'C', 'H'];
const TL_HITTERS: &[char] = &['J', 'X', 'Z', 'K', 'H', 'F', 'W', 'Y', 'V', 'M', 'P', 'C'];
const LENGTHENER_LETTERS: &[char] = &['E', 'R', 'I', 'N', 'G', 'L', 'Y', 'D', 'S'];
const LENGTHENER_TRIADS: &[&[char]] = &[&['I', 'N', 'G'], &['E', 'R', 'S']];
const PROTECTED_PAIRS: &[(char, char)] = &[
    ('C', 'H'),
    ('S', 'H'),
    ('T', 'H'),
    ('P', 'H'),
    ('S', 'T'),
    ('T', 'R'),
    ('P', 'R'),
    ('C', 'R'),
    ('B', 'R'),
    ('D', 'R'),
    ('C', 'L'),
    ('G', 'L'),
    ('P', 'L'),
    ('F', 'R'),
    ('G', 'R'),
    ('S', 'L'),
    ('S', 'N'),
    ('S', 'P'),
    ('Q', 'U'),
];

fn analyze_pass_one(
    letters: &[char],
    target_length: usize,
    baseline_counts: Option<&[u8; 26]>,
) -> PassOneOutcome {
    let rack_len = letters.len();
    let mut keep_flags = vec![true; rack_len];
    let mut notes: Vec<String> = Vec::new();
    let mut focus_tags: Vec<String> = Vec::new();
    let mut desired_letters: Vec<char> = Vec::new();

    let mut positions_by_letter: Vec<Vec<usize>> = vec![Vec::new(); 26];
    for (idx, &ch) in letters.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            positions_by_letter[char_to_index(ch)].push(idx);
        }
    }

    for (letter_idx, positions) in positions_by_letter.iter().enumerate() {
        if positions.len() > 2 {
            let letter = index_to_char(letter_idx);
            let mut extras = positions.len() - 2;
            for &pos in positions.iter().rev() {
                if extras == 0 {
                    break;
                }
                if would_violate_baseline(letter_idx, &keep_flags, &letters, baseline_counts) {
                    continue;
                }
                keep_flags[pos] = false;
                extras = extras.saturating_sub(1);
            }
            if positions.len() > 2 {
                push_note(&mut notes, format!("Trim extra {}", letter));
                push_focus_tag(&mut focus_tags, "Trim duplicates");
            }
        }
    }

    let mut kept_counts = compute_kept_counts(letters, &keep_flags);
    let mut kept_vowels = count_kept_vowels(letters, &keep_flags);

    let q_idx = char_to_index('Q');
    let u_idx = char_to_index('U');
    if kept_counts[q_idx] > 0 && kept_counts[u_idx] == 0 {
        let mut dropped_q = false;
        for &pos in &positions_by_letter[q_idx] {
            if would_violate_baseline(q_idx, &keep_flags, &letters, baseline_counts) {
                continue;
            }
            keep_flags[pos] = false;
            dropped_q = true;
        }
        if dropped_q {
            push_note(&mut notes, "Dump Q (no U)".to_string());
            push_focus_tag(&mut focus_tags, "Fix Q support");
        } else {
            push_unique_char(&mut desired_letters, 'U');
            push_note(
                &mut notes,
                "Need U to unlock your Q for TL/DL plays".to_string(),
            );
            push_focus_tag(&mut focus_tags, "Fix Q support");
        }
        kept_counts = compute_kept_counts(letters, &keep_flags);
        kept_vowels = count_kept_vowels(letters, &keep_flags);
    }

    if !has_core_vowel(letters, &keep_flags) {
        let v_idx = char_to_index('V');
        if kept_counts[v_idx] > 0 {
            for &pos in &positions_by_letter[v_idx] {
                if would_violate_baseline(v_idx, &keep_flags, &letters, baseline_counts) {
                    continue;
                }
                keep_flags[pos] = false;
            }
            push_note(&mut notes, "Drop V until you secure A/E/I/O".to_string());
            push_focus_tag(&mut focus_tags, "Balance vowels");
            kept_counts = compute_kept_counts(letters, &keep_flags);
            kept_vowels = count_kept_vowels(letters, &keep_flags);
        }
    }

    let (vowel_min, vowel_max) = desired_vowel_range(rack_len, target_length);

    if kept_vowels > vowel_max {
        let excess = kept_vowels - vowel_max;
        let mut drop_candidates: Vec<(i32, usize)> = letters
            .iter()
            .enumerate()
            .filter(|(idx, ch)| keep_flags[*idx] && is_vowel(**ch))
            .map(|(idx, &ch)| {
                let base = match ch {
                    'U' => 70,
                    'O' => 55,
                    'A' => 45,
                    'I' => 35,
                    'E' => 25,
                    _ => 30,
                };
                let duplicates = (kept_counts[char_to_index(ch)] as i32 - 1).max(0) * 12;
                (base + duplicates, idx)
            })
            .collect();
        drop_candidates.sort_by(|a, b| b.0.cmp(&a.0));

        let mut trimmed: Vec<char> = Vec::new();
        let mut removed = 0;
        for (_, idx) in drop_candidates.into_iter() {
            if removed >= excess {
                break;
            }
            let ch = letters[idx];
            if would_violate_baseline(char_to_index(ch), &keep_flags, &letters, baseline_counts) {
                continue;
            }
            keep_flags[idx] = false;
            trimmed.push(ch);
            removed += 1;
        }
        if !trimmed.is_empty() {
            push_note(
                &mut notes,
                format!("Trim excess vowels ({})", format_letters(&trimmed)),
            );
            push_focus_tag(&mut focus_tags, "Balance vowels");
        }
        kept_counts = compute_kept_counts(letters, &keep_flags);
        kept_vowels = count_kept_vowels(letters, &keep_flags);
    }

    if kept_vowels < vowel_min {
        let mut needed = vowel_min - kept_vowels;
        let mut drop_candidates: Vec<(i32, usize)> = letters
            .iter()
            .enumerate()
            .filter(|(idx, ch)| keep_flags[*idx] && !is_vowel(**ch))
            .map(|(idx, &ch)| {
                let base = consonant_drop_priority(ch);
                let duplicates = (kept_counts[char_to_index(ch)] as i32 - 1).max(0) * 8;
                (base + duplicates, idx)
            })
            .collect();
        drop_candidates.sort_by(|a, b| b.0.cmp(&a.0));

        let mut dropped_letters: Vec<char> = Vec::new();
        for (_, idx) in drop_candidates {
            if needed == 0 {
                break;
            }
            let ch = letters[idx];
            if ch == 'S' || is_glue_consonant(ch) {
                continue;
            }
            if would_break_protected_pair(ch, &kept_counts) {
                continue;
            }
            if would_violate_baseline(char_to_index(ch), &keep_flags, &letters, baseline_counts) {
                continue;
            }
            keep_flags[idx] = false;
            dropped_letters.push(ch);
            needed = needed.saturating_sub(1);
            let letter_idx = char_to_index(ch);
            kept_counts[letter_idx] = kept_counts[letter_idx].saturating_sub(1);
        }

        if !dropped_letters.is_empty() {
            push_note(
                &mut notes,
                format!(
                    "Reroll {} to fish for stronger vowels",
                    format_letters(&dropped_letters)
                ),
            );
            push_focus_tag(&mut focus_tags, "Add vowels");
        }

        push_unique_char(&mut desired_letters, 'E');
        push_unique_char(&mut desired_letters, 'A');
        push_unique_char(&mut desired_letters, 'I');
        push_note(
            &mut notes,
            "Aim for two reliable vowels (E/A/I)".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Add vowels");
    }

    kept_counts = compute_kept_counts(letters, &keep_flags);

    let preserved_pairs = collect_protected_pairs(&kept_counts);
    if !preserved_pairs.is_empty() {
        push_note(
            &mut notes,
            format!("Preserve blends ({})", preserved_pairs.join(", ")),
        );
        push_focus_tag(&mut focus_tags, "Protect blends");
    }

    let keep_letters: Vec<char> = letters
        .iter()
        .enumerate()
        .filter(|(idx, _)| keep_flags[*idx])
        .map(|(_, &ch)| ch)
        .collect();

    let reroll_letters: Vec<char> = letters
        .iter()
        .enumerate()
        .filter(|(idx, _)| !keep_flags[*idx])
        .map(|(_, &ch)| ch)
        .collect();

    if keep_letters.iter().any(|&ch| ch == 'S') {
        push_note(&mut notes, "Keep S for easy hooks".to_string());
        push_focus_tag(&mut focus_tags, "Keep S hot");
    }
    if keep_letters
        .iter()
        .any(|&ch| is_glue_consonant(ch) && ch != 'S')
    {
        push_note(
            &mut notes,
            "Hold glue consonants (R, T, L, N, D, M, P, C, H)".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Keep glue consonants");
    }
    if keep_letters
        .iter()
        .any(|&ch| matches!(ch, 'J' | 'X' | 'Z' | 'K'))
    {
        push_note(
            &mut notes,
            "Keep one premium hitter ready for TL".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Prep TL hitter");
    }
    if reroll_letters.is_empty() {
        push_note(
            &mut notes,
            "Rack already balanced — optional reroll".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Fine-tune only");
    }

    PassOneOutcome {
        keep_flags,
        keep_letters,
        reroll_letters,
        desired_letters: desired_letters.into_iter().take(8).collect(),
        notes,
        vowel_min,
        focus_tags,
    }
}

fn analyze_pass_two(
    letters: &[char],
    target_length: usize,
    pass_one: &PassOneOutcome,
    baseline_counts: Option<&[u8; 26]>,
) -> Option<RerollAdvice> {
    let mut keep_flags = pass_one.keep_flags.clone();
    let mut notes: Vec<String> = Vec::new();
    let mut focus_tags: Vec<String> = Vec::new();
    let mut desired_letters: Vec<char> = Vec::new();

    let mut keep_counts = compute_kept_counts(letters, &keep_flags);
    let mut current_vowels = count_kept_vowels(letters, &keep_flags);
    let mut current_lengtheners = letters
        .iter()
        .enumerate()
        .filter(|(idx, &ch)| keep_flags[*idx] && is_lengthener_letter(ch))
        .count();
    let lengthener_uniques: HashSet<char> = letters
        .iter()
        .enumerate()
        .filter(|(idx, &ch)| keep_flags[*idx] && is_lengthener_letter(ch))
        .map(|(_, &ch)| ch)
        .collect();
    let mut current_unique_lengtheners = lengthener_uniques.len();
    let mut current_tl_hitters = letters
        .iter()
        .enumerate()
        .filter(|(idx, &ch)| keep_flags[*idx] && is_tl_candidate(ch))
        .count();

    let mut reroll_letters: Vec<char> = pass_one.reroll_letters.clone();

    let has_s = letters
        .iter()
        .enumerate()
        .any(|(idx, &ch)| keep_flags[idx] && ch == 'S');

    if !has_s {
        push_unique_char(&mut desired_letters, 'S');
        push_note(&mut notes, "Look for an S to extend words".to_string());
        push_focus_tag(&mut focus_tags, "Find S hook");
    }

    let mut chase_lengtheners = false;
    let mut protect_lengtheners = false;
    if target_length >= 7 {
        if current_unique_lengtheners < 3 {
            chase_lengtheners = true;
            protect_lengtheners = true;
        }
        for triad in LENGTHENER_TRIADS {
            let mut missing: Vec<char> = triad
                .iter()
                .copied()
                .filter(|ch| !lengthener_uniques.contains(ch))
                .collect();
            if !missing.is_empty() {
                chase_lengtheners = true;
                protect_lengtheners = true;
                missing.sort_unstable();
                for ch in missing {
                    push_unique_char(&mut desired_letters, ch);
                }
            }
        }
    } else if current_lengtheners < 2 {
        chase_lengtheners = true;
        protect_lengtheners = true;
    }

    if chase_lengtheners {
        for &ch in LENGTHENER_LETTERS {
            push_unique_char(&mut desired_letters, ch);
        }
        push_note(
            &mut notes,
            "Chase lengtheners (-ER/-ED/-ING/-LY) to stretch onto DW/TW".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Chase lengtheners");
    }

    if current_tl_hitters == 0 {
        for &ch in TL_HITTERS {
            push_unique_char(&mut desired_letters, ch);
        }
        push_note(
            &mut notes,
            "Fish for a TL hitter (J/X/Z/K/H/F/W/Y)".to_string(),
        );
        push_focus_tag(&mut focus_tags, "Find TL hitter");
    }

    if desired_letters.is_empty() && reroll_letters.is_empty() {
        return None;
    }

    let mut candidates: Vec<(i32, usize, char)> = letters
        .iter()
        .enumerate()
        .filter(|(idx, _)| keep_flags[*idx])
        .map(|(idx, &ch)| {
            let base = if is_vowel(ch) {
                40
            } else {
                consonant_drop_priority(ch)
            };
            let duplicates = (keep_counts[char_to_index(ch)] as i32 - 1).max(0) * 10;
            (base + duplicates, idx, ch)
        })
        .collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0));

    let mut dropped: Vec<char> = Vec::new();
    for (_, idx, ch) in candidates {
        if dropped.len() >= 2 {
            break;
        }
        if !keep_flags[idx] {
            continue;
        }
        if ch == 'S' || is_glue_consonant(ch) {
            continue;
        }
        if would_break_protected_pair(ch, &keep_counts) {
            continue;
        }
        if is_lengthener_letter(ch) {
            if target_length >= 7 {
                if protect_lengtheners || current_unique_lengtheners <= 3 {
                    continue;
                }
            } else if current_lengtheners <= 2 {
                continue;
            }
        }
        if is_tl_candidate(ch) && current_tl_hitters <= 1 {
            continue;
        }
        if is_vowel(ch) && current_vowels <= pass_one.vowel_min {
            continue;
        }
        if would_violate_baseline(char_to_index(ch), &keep_flags, &letters, baseline_counts) {
            continue;
        }

        keep_flags[idx] = false;
        reroll_letters.push(ch);
        dropped.push(ch);

        keep_counts[char_to_index(ch)] = keep_counts[char_to_index(ch)].saturating_sub(1);
        if is_vowel(ch) {
            current_vowels = current_vowels.saturating_sub(1);
        }
        if is_lengthener_letter(ch) {
            current_lengtheners = current_lengtheners.saturating_sub(1);
            if keep_counts[char_to_index(ch)] == 0 {
                current_unique_lengtheners = current_unique_lengtheners.saturating_sub(1);
            }
            if target_length >= 7 {
                if current_unique_lengtheners <= 3 {
                    protect_lengtheners = true;
                }
                if LENGTHENER_TRIADS.iter().any(|triad| {
                    triad
                        .iter()
                        .any(|&triad_ch| keep_counts[char_to_index(triad_ch)] == 0)
                }) {
                    protect_lengtheners = true;
                }
            }
        }
        if is_tl_candidate(ch) {
            current_tl_hitters = current_tl_hitters.saturating_sub(1);
        }
    }

    if !dropped.is_empty() {
        push_note(
            &mut notes,
            format!(
                "Swap {} to upgrade your multiplier letters",
                format_letters(&dropped)
            ),
        );
        push_focus_tag(&mut focus_tags, "Upgrade multipliers");
    }

    let keep_letters: Vec<char> = letters
        .iter()
        .enumerate()
        .filter(|(idx, _)| keep_flags[*idx])
        .map(|(_, &ch)| ch)
        .collect();

    let desired_snapshot = desired_letters.clone();
    let missing_letters: Vec<char> = desired_snapshot.iter().cloned().take(10).collect();
    let probability =
        approximate_draw_probability(&keep_letters, &reroll_letters, &desired_snapshot);

    Some(RerollAdvice {
        target_word: "Pass 2 – Target the board".to_string(),
        missing_letters,
        reroll_letters,
        keep_letters,
        estimated_score: None,
        success_probability: probability,
        phase: "target",
        notes,
        focus_tags,
    })
}

fn desired_vowel_range(rack_len: usize, target_length: usize) -> (usize, usize) {
    if rack_len == 0 {
        return (0, 0);
    }

    if rack_len <= 4 {
        (1.min(rack_len), 2.min(rack_len))
    } else if rack_len == 5 || target_length <= 5 {
        (2.min(rack_len), 2.min(rack_len))
    } else {
        (2.min(rack_len), 3.min(rack_len))
    }
}

fn compute_kept_counts(letters: &[char], keep_flags: &[bool]) -> [u8; 26] {
    let mut counts = [0u8; 26];
    for (idx, &keep) in keep_flags.iter().enumerate() {
        if keep {
            let ch = letters[idx];
            if ch.is_ascii_uppercase() {
                counts[char_to_index(ch)] += 1;
            }
        }
    }
    counts
}

fn count_kept_vowels(letters: &[char], keep_flags: &[bool]) -> usize {
    letters
        .iter()
        .enumerate()
        .filter(|(idx, ch)| keep_flags[*idx] && is_vowel(**ch))
        .count()
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'A' | 'E' | 'I' | 'O' | 'U')
}

fn has_core_vowel(letters: &[char], keep_flags: &[bool]) -> bool {
    letters
        .iter()
        .enumerate()
        .any(|(idx, &ch)| keep_flags[idx] && matches!(ch, 'A' | 'E' | 'I' | 'O'))
}

fn is_glue_consonant(ch: char) -> bool {
    GLUE_CONSONANTS.contains(&ch)
}

fn is_lengthener_letter(ch: char) -> bool {
    LENGTHENER_LETTERS.contains(&ch)
}

fn is_tl_candidate(ch: char) -> bool {
    TL_HITTERS.contains(&ch)
}

fn would_break_protected_pair(ch: char, keep_counts: &[u8; 26]) -> bool {
    for &(a, b) in PROTECTED_PAIRS {
        if ch == a {
            let a_count = keep_counts[char_to_index(a)];
            let b_count = keep_counts[char_to_index(b)];
            if a_count <= 1 && b_count > 0 {
                return true;
            }
        } else if ch == b {
            let a_count = keep_counts[char_to_index(a)];
            let b_count = keep_counts[char_to_index(b)];
            if b_count <= 1 && a_count > 0 {
                return true;
            }
        }
    }
    false
}

fn collect_protected_pairs(keep_counts: &[u8; 26]) -> Vec<String> {
    let mut pairs: Vec<String> = Vec::new();
    for &(a, b) in PROTECTED_PAIRS {
        let a_count = keep_counts[char_to_index(a)];
        let b_count = keep_counts[char_to_index(b)];
        if a_count > 0 && b_count > 0 {
            let pair = format!("{}{}", a, b);
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    pairs
}

fn char_to_index(ch: char) -> usize {
    (ch as u8 - b'A') as usize
}

fn index_to_char(idx: usize) -> char {
    (b'A' + idx as u8) as char
}

fn push_unique_char(target: &mut Vec<char>, ch: char) {
    if !target.contains(&ch) {
        target.push(ch);
    }
}

fn push_note(notes: &mut Vec<String>, note: impl Into<String>) {
    let text = note.into();
    if !notes.contains(&text) {
        notes.push(text);
    }
}

fn push_focus_tag(tags: &mut Vec<String>, tag: impl Into<String>) {
    let text = tag.into();
    if !tags
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&text))
    {
        tags.push(text);
    }
}

fn format_letters(letters: &[char]) -> String {
    letters
        .iter()
        .map(|ch| ch.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

fn consonant_drop_priority(ch: char) -> i32 {
    match ch {
        'Q' => 110,
        'V' => 95,
        'W' => 85,
        'Y' => 80,
        'F' | 'H' => 70,
        'B' | 'G' => 65,
        'K' => 60,
        'J' | 'X' | 'Z' => 55,
        'P' | 'C' | 'M' => 50,
        'D' => 40,
        'R' | 'T' | 'L' | 'N' | 'S' => 15,
        _ => 45,
    }
}

fn approximate_draw_probability(
    keep_letters: &[char],
    reroll_letters: &[char],
    desired_letters: &[char],
) -> Option<f64> {
    if reroll_letters.is_empty() || desired_letters.is_empty() {
        return None;
    }

    let mut bag_counts: [i32; 26] = LETTER_BAG_COUNTS.map(|count| i32::from(count));

    for &ch in keep_letters {
        if ch.is_ascii_uppercase() {
            let idx = char_to_index(ch);
            if bag_counts[idx] > 0 {
                bag_counts[idx] -= 1;
            }
        }
    }

    let draws = reroll_letters.len() as u32;
    if draws == 0 {
        return None;
    }

    let mut desired_indices: Vec<usize> = Vec::new();
    for &ch in desired_letters {
        if ch.is_ascii_uppercase() {
            let idx = char_to_index(ch);
            if !desired_indices.contains(&idx) {
                desired_indices.push(idx);
            }
        }
    }

    if desired_indices.is_empty() {
        return None;
    }

    let mut desired_total: u32 = 0;
    for &idx in &desired_indices {
        if bag_counts[idx] > 0 {
            desired_total += bag_counts[idx] as u32;
        }
    }

    if desired_total == 0 {
        return Some(0.0);
    }

    let total_available: u32 = bag_counts
        .iter()
        .filter(|&&count| count > 0)
        .map(|&count| count as u32)
        .sum();

    if total_available == 0 {
        return None;
    }

    if draws > total_available {
        return Some(1.0);
    }

    let undesired_available = total_available.saturating_sub(desired_total);
    if undesired_available < draws {
        return Some(1.0);
    }

    let total_combos = binomial(total_available, draws);
    if total_combos == 0.0 {
        return None;
    }

    let miss_combos = binomial(undesired_available, draws);
    let success = 1.0 - (miss_combos / total_combos);
    Some(success.clamp(0.0, 1.0))
}

fn binomial(n: u32, k: u32) -> f64 {
    if k > n {
        return 0.0;
    }
    if k == 0 || k == n {
        return 1.0;
    }
    let k = k.min(n - k);
    let mut result = 1.0f64;
    for i in 1..=k {
        result *= f64::from(n - k + i);
        result /= f64::from(i);
    }
    result
}

fn letter_counts_str(word: &str) -> [u8; 26] {
    let mut counts = [0u8; 26];
    for ch in word.chars() {
        if ch.is_ascii_alphabetic() {
            let idx = char_to_index(ch.to_ascii_uppercase());
            counts[idx] = counts[idx].saturating_add(1);
        }
    }
    counts
}

fn would_violate_baseline(
    letter_idx: usize,
    keep_flags: &[bool],
    letters: &[char],
    baseline_counts: Option<&[u8; 26]>,
) -> bool {
    let required = baseline_counts
        .and_then(|counts| counts.get(letter_idx))
        .copied()
        .unwrap_or(0);
    if required == 0 {
        return false;
    }
    let mut current = 0u8;
    for (keep, &ch) in keep_flags.iter().zip(letters.iter()) {
        if *keep && ch.is_ascii_alphabetic() && char_to_index(ch) == letter_idx {
            current = current.saturating_add(1);
        }
    }
    current <= required
}

fn score_word_with_bonuses(word: &str, bonuses: &[Bonus], round_multiplier: u32) -> Option<u32> {
    let mut sum: u32 = 0;
    let mut word_multiplier: u32 = 1;

    for (idx, ch) in word.chars().enumerate() {
        let base = u32::from(scoring::letter_value(ch)?);
        let bonus = bonuses.get(idx).copied().unwrap_or(Bonus::None);
        sum += base * bonus.letter_multiplier();
        word_multiplier = word_multiplier.saturating_mul(bonus.word_multiplier());
    }

    Some(
        sum.saturating_mul(word_multiplier)
            .saturating_mul(round_multiplier),
    )
}

fn word_fits(word: &str, rack_counts: &[u8; 26]) -> bool {
    let mut need = [0u8; 26];
    for ch in word.chars() {
        if !ch.is_ascii_alphabetic() {
            return false;
        }
        let idx = (ch as u8 - b'A') as usize;
        need[idx] += 1;
        if need[idx] > rack_counts[idx] {
            return false;
        }
    }
    true
}

fn letter_counts(letters: &[char]) -> [u8; 26] {
    let mut counts = [0u8; 26];
    for ch in letters {
        if ch.is_ascii_alphabetic() {
            let idx = (*ch as u8 - b'A') as usize;
            counts[idx] += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pass_two_chases_lengthener_triads_for_long_targets() {
        let rack: Vec<char> = "ABCDINT".chars().collect();
        let advice = suggest_rerolls(&rack, 7, &HashSet::new(), 3, None);
        let pass_two = advice
            .iter()
            .find(|entry| entry.phase == "target")
            .expect("expected pass-two advice");

        assert!(pass_two.missing_letters.contains(&'G'));
        assert!(pass_two.missing_letters.contains(&'E'));
        assert!(!pass_two
            .notes
            .iter()
            .any(|note| note.contains("Rack already balanced")));
    }
}
