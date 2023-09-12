
#[derive(Default, Copy, Clone)]
pub struct Counters {
    pub writes: u32,
    pub reads: u32,
    pub breaks: u32,
    pub collisions: u32,
    pub exactreads: u32,
    pub iters: u32
}