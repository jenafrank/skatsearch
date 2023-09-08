
#[derive(Default, Copy, Clone)]
pub struct Counters {
    pub cnt_writes: u32,
    pub cnt_reads: u32,
    pub cnt_breaks: u32,
    pub cnt_collisions: u32,
    pub cnt_exactreads: u32,
    pub cnt_iters: u32
}