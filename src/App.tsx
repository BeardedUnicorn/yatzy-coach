import type { JSX } from "react";
import { ChangeEvent, useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const RACK_SIZE = 7;
const WORD_TRACKER_STORAGE_KEY = "yatzy-coach::word-tracker";
type BonusOption = "NONE" | "DL" | "TL" | "DW" | "TW";
const BONUS_OPTIONS: BonusOption[] = ["NONE", "DL", "TL", "DW", "TW"];

type RoundOption = {
  value: string;
  label: string;
};

const ROUND_OPTIONS: RoundOption[] = [
  { value: "1", label: "Round 1 (×1)" },
  { value: "2", label: "Round 2 (×2)" },
  { value: "3", label: "Round 3 (×3)" },
  { value: "4", label: "Round 4 (×4)" },
  { value: "5", label: "Round 5 (×5)" },
];

const BONUS_LABELS: Record<BonusOption, string> = {
  NONE: "None",
  DL: "DL",
  TL: "TL",
  DW: "DW",
  TW: "TW",
};

type WordRecommendation = {
  slot_index?: number | null;
  word: string;
  score?: number | null;
  computed_score?: number | null;
  confidence?: number | null;
  letters_used: string[];
  placement_notes?: string | null;
};

type RerollSuggestion = {
  target_word: string;
  missing_letters: string[];
  reroll_letters: string[];
  keep_letters: string[];
  estimated_score?: number | null;
  success_probability?: number | null;
  phase?: string | null;
  notes?: string[];
};

type SolveRackResponse = {
  rack_letters: string[];
  target_word_length?: number | null;
  rack_bonuses?: BonusOption[];
  round?: number | null;
  recommendations: WordRecommendation[];
  reroll_suggestions?: RerollSuggestion[];
};

const BackspaceIcon = (): JSX.Element => (
  <svg
    viewBox="0 0 24 24"
    width="20"
    height="20"
    xmlns="http://www.w3.org/2000/svg"
    focusable="false"
    aria-hidden="true"
  >
    <path
      d="M9.2 19a1 1 0 0 1-.78-.38l-5-6a1 1 0 0 1 0-1.24l5-6A1 1 0 0 1 9.2 5H19a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H9.2Zm.43-2H19V7H9.63l-4 5ZM12.7 15.3a1 1 0 0 1 0-1.4l1.3-1.3-1.3-1.3a1 1 0 1 1 1.4-1.4l1.3 1.3 1.3-1.3a1 1 0 1 1 1.4 1.4l-1.3 1.3 1.3 1.3a1 1 0 0 1-1.4 1.4l-1.3-1.3-1.3 1.3a1 1 0 0 1-1.4 0Z"
      fill="currentColor"
    />
  </svg>
);

const ClearIcon = (): JSX.Element => (
  <svg
    viewBox="0 0 24 24"
    width="20"
    height="20"
    xmlns="http://www.w3.org/2000/svg"
    focusable="false"
    aria-hidden="true"
  >
    <path
      d="M5.5 19a1 1 0 0 1-.92-1.39l5-12A1 1 0 0 1 10.5 5h5a1 1 0 0 1 .92.61l2 5a1 1 0 0 1-1.84.74L14.72 7h-3.68l-4.27 10.28A1 1 0 0 1 5.5 19ZM9 21a1 1 0 0 1-.7-1.71l6-6a1 1 0 0 1 1.4 1.42l-6 6A1 1 0 0 1 9 21Zm8 0a1 1 0 0 1-.7-.29l-6-6a1 1 0 0 1 1.4-1.42l6 6A1 1 0 0 1 17 21Z"
      fill="currentColor"
    />
  </svg>
);

const App = () => {
  const [rackText, setRackText] = useState("");
  const [targetLength, setTargetLength] = useState("");
  const [rackBonuses, setRackBonuses] = useState<BonusOption[]>(
    Array.from({ length: RACK_SIZE }, () => "NONE"),
  );
  const [round, setRound] = useState<string>("1");
  const [result, setResult] = useState<SolveRackResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [invalidWords, setInvalidWords] = useState<string[]>([]);

  useEffect(() => {
    const stored = window.localStorage.getItem(WORD_TRACKER_STORAGE_KEY);
    if (!stored) return;

    try {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed)) {
        const normalized = parsed
          .filter((entry): entry is string => typeof entry === "string")
          .map((entry) => entry.toUpperCase().trim())
          .filter((entry) => entry.length > 0);
        setInvalidWords(Array.from(new Set(normalized)));
      } else if (parsed && Array.isArray(parsed.invalidWords)) {
        const normalized = parsed.invalidWords
          .filter((entry: unknown): entry is string => typeof entry === "string")
          .map((entry: string) => entry.toUpperCase().trim())
          .filter((entry: string) => entry.length > 0);
        setInvalidWords(Array.from(new Set(normalized)));
      }
    } catch (err) {
      console.warn("Unable to parse stored words", err);
    }
  }, []);

  useEffect(() => {
    window.localStorage.setItem(
      WORD_TRACKER_STORAGE_KEY,
      JSON.stringify(invalidWords),
    );
  }, [invalidWords]);

  const rackLetters = useMemo(() => rackText.split(""), [rackText]);

  const rackSlots = useMemo(
    () =>
      Array.from({ length: RACK_SIZE }, (_, index) => rackLetters[index] ?? ""),
    [rackLetters],
  );

  const normalizedInvalidSet = useMemo(() => {
    const set = new Set<string>();
    invalidWords.forEach((word) => set.add(word.toUpperCase().trim()));
    return set;
  }, [invalidWords]);

  const handleRackInputChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const sanitized = event.target.value
        .toUpperCase()
        .replace(/[^A-Z]/g, "")
        .slice(0, RACK_SIZE);
      setRackText(sanitized);
      setError(null);
    },
    [],
  );

  const handleBackspace = useCallback(() => {
    setRackText((prev) => prev.slice(0, -1));
    setError(null);
  }, []);

  const handleClearRack = useCallback(() => {
    setRackText("");
    setResult(null);
    setError(null);
    setRackBonuses(Array.from({ length: RACK_SIZE }, () => "NONE"));
  }, []);

  const handleTargetLengthChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setTargetLength(event.target.value);
      setError(null);
    },
    [],
  );

  const parseTargetLength = useCallback((): number | null => {
    const trimmed = targetLength.trim();
    if (!trimmed) return null;
    const parsed = Number.parseInt(trimmed, 10);
    if (Number.isNaN(parsed)) {
      throw new Error("Target length must be a number.");
    }
    return parsed;
  }, [targetLength]);

  const handleBonusChange = useCallback((slot: number, value: BonusOption) => {
    setRackBonuses((prev) => {
      const next = [...prev];
      next[slot] = value;
      return next;
    });
  }, []);

  const handleRoundChange = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setRound(event.target.value);
      setError(null);
    },
    [],
  );

  const handleSolve = useCallback(async () => {
    let targetValue: number | null = null;

    try {
      targetValue = parseTargetLength();
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Unable to read the target length.");
      }
      return;
    }

    if (rackLetters.length === 0) {
      setError("Add at least one rack letter before solving.");
      return;
    }

    setError(null);
    setIsLoading(true);

    try {
      const parsedRound = Number.parseInt(round, 10);
      if (Number.isNaN(parsedRound) || parsedRound < 1 || parsedRound > 5) {
        throw new Error("Select a valid round between 1 and 5.");
      }

      const payload = {
        rack_letters: rackLetters,
        target_word_length: targetValue,
        invalid_words: invalidWords,
        rack_bonuses: rackBonuses,
        round: parsedRound,
      };

      const response = await invoke<SolveRackResponse>("solve_rack_command", {
        request: payload,
      });

      const filtered: SolveRackResponse = {
        ...response,
        recommendations: response.recommendations.filter(
          (rec) => !normalizedInvalidSet.has(rec.word.toUpperCase().trim()),
        ),
      };

      if (
        Array.isArray(response.rack_bonuses) &&
        response.rack_bonuses.length === RACK_SIZE
      ) {
        setRackBonuses(response.rack_bonuses);
      }

      setResult(filtered);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      } else if (typeof err === "string") {
        setError(err);
      } else {
        setError("Unexpected error occurred while solving the rack.");
      }
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  }, [
    normalizedInvalidSet,
    parseTargetLength,
    rackLetters,
    rackBonuses,
    invalidWords,
    round,
  ]);

  const handleMarkInvalid = useCallback((word: string) => {
    const normalized = word.toUpperCase().trim();
    if (!normalized) return;

    setInvalidWords((prev) =>
      prev.includes(normalized) ? prev : [...prev, normalized],
    );

    setResult((prev) => {
      if (!prev) return prev;
      const filteredRerolls = prev.reroll_suggestions
        ? prev.reroll_suggestions.filter(
            (suggestion) =>
              suggestion.target_word.toUpperCase().trim() !== normalized,
          )
        : prev.reroll_suggestions;
      return {
        ...prev,
        recommendations: prev.recommendations.filter(
          (rec) => rec.word.toUpperCase().trim() !== normalized,
        ),
        reroll_suggestions: filteredRerolls,
      };
    });
  }, []);

  const handleRemoveInvalidWord = useCallback((word: string) => {
    const normalized = word.toUpperCase().trim();
    if (!normalized) return;
    setInvalidWords((prev) =>
      prev.filter((existing) => existing !== normalized),
    );
  }, []);

  useEffect(() => {
    if (!result) return;
    if (!invalidWords.length) return;

    setResult((prev) => {
      if (!prev) return prev;
      const filteredRecommendations = prev.recommendations.filter(
        (rec) => !normalizedInvalidSet.has(rec.word.toUpperCase().trim()),
      );
      const filteredRerolls = prev.reroll_suggestions
        ? prev.reroll_suggestions.filter(
            (suggestion) =>
              !normalizedInvalidSet.has(
                suggestion.target_word.toUpperCase().trim(),
              ),
          )
        : prev.reroll_suggestions;
      const recommendationsUnchanged =
        filteredRecommendations.length === prev.recommendations.length;
      const rerollsUnchanged =
        filteredRerolls === prev.reroll_suggestions ||
        (filteredRerolls?.length ?? 0) ===
          (prev.reroll_suggestions?.length ?? 0);
      if (recommendationsUnchanged && rerollsUnchanged) {
        return prev;
      }

      return {
        ...prev,
        recommendations: filteredRecommendations,
        reroll_suggestions: filteredRerolls,
      };
    });
  }, [normalizedInvalidSet, result, invalidWords]);

  const canSolve = rackLetters.length > 0 && !isLoading;
  const hasRecommendations = (result?.recommendations?.length ?? 0) > 0;
  const hasReroll = (result?.reroll_suggestions?.length ?? 0) > 0;
  const resultsLayoutClass = hasReroll ? "results-body two-column" : "results-body";

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <h1>Word Yatzy Coach</h1>
          <p className="subtitle">
            Build your rack, set a target length, and find the highest scoring words.
          </p>
        </div>
      </header>

      <section className="panel rack-builder">
        <h2>Rack Builder</h2>
        <p className="helper-text">
          Type your rack letters directly. The rack holds up to seven tiles, and each slot can be
          given a multiplier to reflect bag bonuses.
        </p>

          <div className="rack-controls">
            <label className="field">
              <span>Rack letters</span>
              <input
                value={rackText}
              onChange={handleRackInputChange}
              placeholder="ENTER7"
              maxLength={RACK_SIZE}
            />
          </label>

          <div className="rack-display">
            {rackSlots.map((letter, index) => (
              <div className="rack-slot" key={`slot-${index}`}>
                <span>{letter || "\u00A0"}</span>
                <select
                  value={rackBonuses[index]}
                  onChange={(event) =>
                    handleBonusChange(index, event.target.value as BonusOption)
                  }
                >
                  {BONUS_OPTIONS.map((option) => (
                    <option key={option} value={option}>
                      {BONUS_LABELS[option]}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>

          <div className="rack-actions">
            <button
              type="button"
              className="secondary"
              onClick={handleBackspace}
              disabled={rackText.length === 0}
              aria-label="Backspace"
              title="Backspace"
            >
              <BackspaceIcon />
            </button>
            <button
              type="button"
              className="secondary"
              onClick={handleClearRack}
              disabled={rackText.length === 0}
              aria-label="Clear rack"
              title="Clear rack"
            >
              <ClearIcon />
            </button>
          </div>

          <div className="solve-controls">
            <label className="field target-length">
              <span>Target word length (optional)</span>
              <input
                value={targetLength}
                onChange={handleTargetLengthChange}
                placeholder="e.g. 5"
                inputMode="numeric"
              />
            </label>

            <label className="field round-select">
              <span>Round</span>
              <select value={round} onChange={handleRoundChange}>
                {ROUND_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>

            <button
              type="button"
              className="primary"
              onClick={handleSolve}
              disabled={!canSolve}
            >
              {isLoading ? "Solving..." : "Solve Rack"}
            </button>
          </div>
        </div>
      </section>

      {error && (
        <section className="panel error">
          <h2>Solver Error</h2>
          <p>{error}</p>
        </section>
      )}

      {result && (
        <section className="panel results">
          <div className="results-header">
            <h2>Top Words</h2>
            <div className="rack-summary">
              <span>Rack: {result.rack_letters.join(" ") || "—"}</span>
              {result.target_word_length ? (
                <span>Target length: {result.target_word_length}</span>
              ) : (
                <span>Any length</span>
              )}
              {result.round ? (
                <span>Round: {result.round} (×{result.round})</span>
              ) : null}
            </div>
          </div>

          <div className={resultsLayoutClass}>
            <div className="results-list">
              {result.recommendations.length === 0 ? (
                <p className="muted">No playable words found for this rack.</p>
              ) : (
                <ul className="recommendations">
                  {result.recommendations.map((rec, idx) => (
                    <li key={`${rec.word}-${idx}`}>
                      <div className="word-line">
                        <span className="word">{rec.word}</span>
                        <span className="score">
                          Score: {rec.computed_score ?? rec.score ?? "—"}
                        </span>
                      </div>
                      <div className="meta">
                        <span>Uses: {rec.letters_used.join(", ")}</span>
                      </div>
                      <div className="word-actions">
                        <button
                          type="button"
                          className="secondary"
                          onClick={() => handleMarkInvalid(rec.word)}
                        >
                          Block Word
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {hasReroll && result.reroll_suggestions && (
              <div className="reroll">
                <h3>Reroll Guidance</h3>
                <p>
                  {hasRecommendations
                    ? "These swaps can push for a higher-scoring multiplier play."
                    : "Swap the suggested letters to chase the target word length and multipliers."}
                </p>
                <ul>
                  {result.reroll_suggestions.map((suggestion, idx) => {
                    const chance =
                      suggestion.success_probability != null
                        ? Math.round(
                            Math.max(
                              0,
                              Math.min(1, suggestion.success_probability),
                            ) * 100,
                          )
                        : null;
                    return (
                      <li key={`${suggestion.target_word}-${idx}`}>
                        <div className="word-line">
                          <span className="word">{suggestion.target_word}</span>
                          {suggestion.estimated_score != null && (
                            <span className="score">
                              Est. score: {suggestion.estimated_score}
                            </span>
                          )}
                        </div>
                        <div className="meta">
                          {suggestion.phase && (
                            <span>
                              Phase:{" "}
                              {suggestion.phase
                                .split("-")
                                .map(
                                  (part) =>
                                    `${part.charAt(0).toUpperCase()}${part
                                      .slice(1)
                                      .toLowerCase()}`,
                                )
                                .join(" ")}
                            </span>
                          )}
                          <span>
                            Missing:{" "}
                            {suggestion.missing_letters.length > 0
                              ? suggestion.missing_letters.join(", ")
                              : "—"}
                          </span>
                          <span>
                            Keep:{" "}
                            {suggestion.keep_letters.length > 0
                              ? suggestion.keep_letters.join(", ")
                              : "—"}
                          </span>
                          <span>
                            Reroll:{" "}
                            {suggestion.reroll_letters.length > 0
                              ? suggestion.reroll_letters.join(", ")
                              : "—"}
                          </span>
                          {chance != null && <span>Chance: {chance}%</span>}
                        </div>
                        {suggestion.notes && suggestion.notes.length > 0 && (
                          <ul className="note-list">
                            {suggestion.notes.map((note, noteIdx) => (
                              <li key={`${suggestion.target_word}-note-${noteIdx}`}>
                                {note}
                              </li>
                            ))}
                          </ul>
                        )}
                      </li>
                    );
                  })}
                </ul>
              </div>
            )}
          </div>
        </section>
      )}

      <section className="panel tracker-panel">
        <h2>Word Tracking</h2>
        <div className="tracker">
          <div>
            <h4>Blocked Words</h4>
            {invalidWords.length === 0 ? (
              <p className="muted">No blocked words yet.</p>
            ) : (
              <ul>
                {invalidWords.map((word) => (
                  <li key={`invalid-${word}`}>
                    <span>{word}</span>
                    <button
                      type="button"
                      onClick={() => handleRemoveInvalidWord(word)}
                    >
                      Remove
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </section>
    </div>
  );
};

export default App;
