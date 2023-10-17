struct Table {
    // fields go here
}

impl Table {
    fn new() -> Table {
        Table {
            // initialize fields here
        }
    }
}

static mut INSTANCE: Option<Table> = None;

fn get_instance() -> &'static mut Table {
    unsafe {
        INSTANCE.get_or_insert_with(|| Table::new())
    }
}