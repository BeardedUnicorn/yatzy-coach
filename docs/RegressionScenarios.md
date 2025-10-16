# Regression Scenarios

## Solver keeps chasing a third vowel for 7-letter turns
- **Setup:** 7-letter rack such as `A C D G L R T`, target length set to 7, baseline word empty.
- **Expectation:** Pass 1 keeps the lone `A`, flags the rack for "Lock three reliable vowels" and asks for additional vowels (E/A/I/O).
- **Reason:** The solver now enforces a three-vowel floor whenever both rack size and target length are ≥7, so sparse racks should funnel rerolls into securing that extra vowel before chasing premiums.
