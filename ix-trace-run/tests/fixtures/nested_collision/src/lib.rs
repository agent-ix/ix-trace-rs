use ix_trace_rs::trace;

#[test]
fn host() {
    #[trace("TC-901")]
    #[test]
    fn collision() {}
}

#[trace("TC-901")]
#[test]
fn collision() {}

#[test]
fn separate_blocks() {
    {
        #[trace("TC-902")]
        fn check() {}
        check();
    }
    {
        #[trace("TC-902")]
        fn check() {}
        check();
    }
}

#[test]
fn separate_blocks_distinct_ids() {
    {
        #[trace("TC-903")]
        fn other() {}
        other();
    }
    {
        #[trace("TC-904")]
        fn other() {}
        other();
    }
}
