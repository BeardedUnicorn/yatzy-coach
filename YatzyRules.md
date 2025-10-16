# Word Yatzy (Android)

A concise, practical reference for **rules, gameplay flow, scoring, and letter (score) values**.

---

## What it is

**Word Yatzy** (aka *Yatzy – the word version*) is a turn‑based mobile word game that blends Scrabble‑style board bonuses with Yahtzee‑style "rolling" (rerolling/swapping) of letter tiles. Matches are quick and played over **5 rounds** against an opponent (PVP/bot). Your goal is to score more total points by building words on a bonus board and managing your letters.

---

## Objective

Score the **most total points after 5 rounds** by placing high‑value words on premium board spaces and using your rerolls efficiently. Filling **all 5 word slots in a round** awards a large **+100 bonus**.

---

## Anatomy of a round

- You begin a round with **7 letters**.
- Each round presents **five word slots** (think of them as five little boards/turns inside the round).
- For each slot you:
  1. **Make a word** from your current 7 letters.
  2. **Optionally reroll/swap** any subset of letters (like dice) before you lock the word for that slot.
  3. **Place the word** on a small grid that contains **letter and word multipliers** (e.g., *double letter*, *triple letter*, *double word*, *triple word*).
  4. **Submit** the slot and move to the next.
- **Fill all five slots in the same round** to bank a **+100 “Yatzy” bonus.**

> Tip: Later rounds are worth more (see **Round multipliers**)—save your best scoring opportunities for Rounds 4–5 when possible.

---

## Scoring

Final score is the sum across all slots and rounds:

**Per‑slot scoring formula**

```
Base word points (sum of letter values after DL/TL)
→ apply word multipliers (DW/TW)
→ multiply by round multiplier (see below)
→ add any per‑round completion bonus (+100 if you filled all 5 slots)
```

### Round multipliers

- **Round 1:** ×1
- **Round 2:** ×2
- **Round 3:** ×3
- **Round 4:** ×4
- **Round 5:** ×5

> Practical effect: a short word with a big letter on a **TL** in Round 5 can outscore a long word in early rounds.

### Bonuses

- **Fill all 5 word slots in a round:** **+100** bonus added once per completed round.

---

## Letter (score) values

Word Yatzy uses familiar **Scrabble‑style** base letter points. Use this table for quick math before multipliers and round bonuses:

**1 point:** A, E, I, L, N, O, R, S, T, U
**2 points:** D, G
**3 points:** B, C, M, P
**4 points:** F, H, V, W, Y
**5 points:** K
**8 points:** J, X
**10 points:** Q, Z

*(No blanks; if you see a blank in special modes, it’s 0 points.)*

---

## Valid words

- Standard English dictionary (proper nouns, abbreviations, and some slang often disallowed). If the game rejects a word, it doesn’t score.
- Letters can be used **once per slot** (they reset for the next slot/round based on your rerolls).

---

## Rerolling / swapping

- Before locking a slot, you may **swap/reroll any subset** of your 7 letters to chase better letters or to fish for a premium‑letter placement.
- Common tactic: **hold** power letters (Q, Z, J, X, K) and reroll the rest until you can park them on **TL**/**DW**/**TW**.

---

## Quick strategy

- **Play the board, not just the rack.** Seek **TL under J/X/Q/Z**, then push the word through a **DW/TW** if possible.
- **Front‑load long words in early rounds?** Only if you can’t engineer premiums. Otherwise, **save big letters** for **Rounds 4–5**.
- **Secure the +100**: when a round is closing, prioritize a simple valid word to fill the **5th slot**.
- Keep a shortlist of reliable **power‑letter minis** for late rounds: *ZA, QI, JO, AX, EX, XI, XU, KA, KI, QAT, QIN*.

---

## Worked examples

1. **Round 3 (×3)**: you place **JAM** with J on a **TL** and no word multiplier.

- Base letters: J(8) + A(1) + M(3) = **12**
- TL under J: 8 → **24**; new sum = 24 + 1 + 3 = **28**
- No DW/TW. Apply round ×3 → **84** points.

2. **Round 5 (×5)**: you place **QAT** with Q on **TL** and whole word on **DW**.

- Base letters: Q(10)+A(1)+T(1)= **12**
- TL under Q: 10 → **30**; sum = 30 + 1 + 1 = **32**
- DW → **64**; Round ×5 → **320** points.

---

## Glossary

- **DL/TL** – Double/Triple Letter; applied to a single letter tile before word/round multipliers.
- **DW/TW** – Double/Triple Word; applied to the whole word after letter multipliers, before round multiplier.
- **Slot** – One word placement within a round (there are five per round).

---

## Table of letter values (copy‑friendly)

```
A1 B3 C3 D2 E1 F4 G2 H4 I1 J8 K5 L1 M3
N1 O1 P3 Q10 R1 S1 T1 U1 V4 W4 X8 Y4 Z10
```

