#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    #[trace("TC-110", "FR-110-AC-1")]
    #[test]
    fn t() {}
}
