use ix_trace_rs::string_edge;

#[string_edge(anything)]
fn edge(_kind: &str) -> bool {
    true
}

fn main() {
    // The item is retained, so this call site must not add a second error.
    let _ = edge("x");
}
