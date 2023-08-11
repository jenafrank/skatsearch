use std::time::Instant;

use crate::types::tt_entry::TtEntry;
use crate::types::tt_flag::TtFlag;
use crate::types::tt_table::TtTable;
use crate::consts::general::TT_SIZE;

impl TtTable {
    pub fn create() -> TtTable {

        let sw = Instant::now();

        let mut x = TtTable {
            data: Vec::with_capacity(TT_SIZE)
        };

        for _ in 0..TT_SIZE {
            x.data.push(TtEntry {
                occupied: false,
                player: Default::default(),
                cards: 0,
                value: 0,
                flag: TtFlag::EXACT,
                trickwon: None,
                bestcard: 0
            });
        }

        // println!("TT creation time = {} µs", sw.elapsed().as_micros());

        x
    }
}
