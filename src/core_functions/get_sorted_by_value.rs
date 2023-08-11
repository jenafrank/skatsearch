use crate::consts::bitboard::JACKOFCLUBS;
use crate::consts::general::SORT_AUGENLIST;

/// Nur für Nicht-Ausspiel-Fall (Kupferschmid: nutze höchste Karte für Zugebung wegen cutoffs)
pub fn get_sorted_by_value(moves: u32) -> ([u32; 10], usize) {

    let mut card = JACKOFCLUBS;
    let mut indexes: Vec<usize> = Vec::new();

    let mut i = 0;
    while card > 0 {
        if moves & card > 0 {
            indexes.push(i);
        }
        card >>= 1;
        i+=1;
    }

    indexes.sort_by(|a,b| SORT_AUGENLIST[*b].partial_cmp(&SORT_AUGENLIST[*a]).unwrap() );

    let mut ordered= [0u32; 10];

    i = 0;
    for el in indexes {
        ordered[i] = JACKOFCLUBS >> el;
        i+=1;
    }

    (ordered, i)
}