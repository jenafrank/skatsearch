/// Nur für Nicht-Ausspiel-Fall (Kupferschmid: nutze höchste Karte für Zugebung wegen cutoffs)
pub fn get_sorted_by_value(moves: u32) -> ([u32; 10], usize) {
    // Pre-sorted indices based on SORT_AUGENLIST values (descending)
    // SORT_AUGENLIST:
    // Indices 0..3 (Jacks) -> Values 16, 15, 14, 13
    // Indices 4, 11, 18, 25 (Aces) -> Value 11
    // Indices 5, 12, 19, 26 (Tens) -> Value 10
    // Indices 6, 13, 20, 27 (Kings) -> Value 4
    // Indices 7, 14, 21, 28 (Queens) -> Value 3
    // Others -> Value 0
    const SORTED_INDICES: [usize; 32] = [
        0, 1, 2, 3, // Jacks
        4, 11, 18, 25, // Aces
        5, 12, 19, 26, // Tens
        6, 13, 20, 27, // Kings
        7, 14, 21, 28, // Queens
        8, 9, 10, 15, 16, 17, 22, 23, 24, 29, 30, 31, // Others (9, 8, 7)
    ];

    let mut ordered = [0u32; 10];
    let mut i = 0;

    for &idx in &SORTED_INDICES {
        // Map index to card bitmask. Index 0 corresponds to JACKOFCLUBS.
        // JACKOFCLUBS is 1 << 31.
        // So card = 1 << (31 - idx).
        let card = 1u32 << (31 - idx);

        if (moves & card) != 0 {
            ordered[i] = card;
            i += 1;
        }
    }

    (ordered, i)
}
