use ix_trace_rs::trace;

#[trace("TC-5b1")]
#[test]
fn from_b() {
    // Tell the harness test which binary this is, when asked.
    if let Some(dir) = std::env::var_os("IXTRACE_EXE_OUT") {
        let exe = std::env::current_exe().expect("exe path");
        std::fs::write(std::path::Path::new(&dir).join("b"), exe.to_string_lossy().as_bytes())
            .expect("write exe path");
    }
}
