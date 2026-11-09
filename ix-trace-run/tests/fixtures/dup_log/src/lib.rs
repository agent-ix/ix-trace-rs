//! Fixture: a test that writes its own result line into the harness log, so
//! the log holds two lines for it.

use std::io::Write;

use ix_trace_rs::trace;

#[trace("TC-600")]
#[test]
fn writes_its_line_twice() {
    let args: Vec<String> = std::env::args().collect();
    let at = args.iter().position(|a| a == "--logfile").expect("run under a log file");
    let mut log = std::fs::OpenOptions::new().append(true).open(&args[at + 1]).expect("log opens");
    log.write_all(b"ok writes_its_line_twice\nok writes_its_line_twice\n").expect("log writes");
}
