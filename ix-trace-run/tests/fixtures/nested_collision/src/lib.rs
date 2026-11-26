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
