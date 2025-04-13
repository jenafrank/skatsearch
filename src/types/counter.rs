#[derive(Default, Debug, Clone, Copy)]

pub struct Counters {
    pub writes: u32,
    pub reads: u32,
    pub breaks: u32,
    pub collisions: u32,
    pub exactreads: u32,
    pub iters: u32,
}

impl Counters {    

    pub fn new() -> Counters {
        Counters::default()
    }

    pub fn inc_writes(&mut self) {
        self.writes += 1;
    }

    pub fn inc_reads(&mut self) {
        self.reads += 1;
    }

    pub fn inc_breaks(&mut self) {
        self.breaks += 1;
    }  

    pub fn inc_collisions(&mut self) {
        self.collisions += 1;
    }   

    pub fn inc_exactreads(&mut self) {
        self.exactreads += 1;
    }   

    pub fn inc_iters(&mut self) {
        self.iters += 1;
    }
    
    pub fn accumulate(all_counters_result: Vec<Counters>) -> Counters {
        
        let mut ret = Counters::new();

        for counter in all_counters_result {
            ret.breaks += counter.breaks;
            ret.collisions += counter.collisions;
            ret.exactreads += counter.exactreads;
            ret.iters += counter.iters;
            ret.writes += counter.writes;
            ret.reads += counter.reads;
        }

        ret
    }
    
    pub fn add(&mut self, counter: Counters) {
        self.breaks += counter.breaks;
        self.collisions += counter.collisions;
        self.exactreads += counter.exactreads;
        self.iters += counter.iters;
        self.writes += counter.writes;
        self.reads += counter.reads;
    }   

}
