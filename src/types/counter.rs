#[derive(Default, Debug, Clone, Copy)]
pub struct Counters {
    pub writes: u32,
    pub reads: u32,
    pub breaks: u32,
    pub collisions: u32,
    pub exactreads: u32,
    pub iters: u32,
}

static COUNTERS_DEFAULT: Counters = Counters {
    writes: 0,
    reads: 0,
    breaks: 0,
    collisions: 0,
    exactreads: 0,
    iters: 0,
};

static mut COUNTERS_INSTANCE: Counters = COUNTERS_DEFAULT;

impl Counters {    

    pub fn new() -> Counters {
        COUNTERS_DEFAULT
    }

    pub fn reset() {
        unsafe { COUNTERS_INSTANCE = COUNTERS_DEFAULT };
    }

    pub fn inc_writes() {
        unsafe { COUNTERS_INSTANCE.writes += 1 };
    }

    pub fn inc_reads() {
        unsafe { COUNTERS_INSTANCE.reads += 1 };
    }

    pub fn inc_breaks() {
        unsafe { COUNTERS_INSTANCE.breaks += 1 };
    }  

    pub fn inc_collisions() {
        unsafe { COUNTERS_INSTANCE.collisions += 1 };
    }   

    pub fn inc_exactreads() {
        unsafe { COUNTERS_INSTANCE.exactreads += 1 };
    }   

    pub fn inc_iters() {
        unsafe { COUNTERS_INSTANCE.iters += 1 };
    }   

    pub fn get() -> Counters {
        unsafe { COUNTERS_INSTANCE }
    }

}
