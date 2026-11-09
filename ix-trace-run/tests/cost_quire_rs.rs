//! TC-022 (FR-006-AC-12, FR-006-CON-1): the runner's wall-clock cost over the
//! quire-rs suite against stock `cargo test --tests`.
//!
//! A benchmark, not a unit test: it needs a quire-rs checkout whose
//! `ix-trace-rs` dev-dependency has `emit` on, and takes as long as the suite
//! does eleven times. Run it with
//! `QUIRE_RS_DIR=<checkout> cargo test -p ix-trace-run --release --test cost_quire_rs -- --ignored --nocapture`.
//! `CARGO_TARGET_DIR` and the profile settings apply to the quire-rs build.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use ix_trace_rs::trace;

/// Alternating pairs of runs when `QUIRE_RS_ROUNDS` is unset (AC-12 asks for
/// at least five).
const DEFAULT_ROUNDS: usize = 5;
/// The most the runner may add: 105% of stock (CON-1).
const LIMIT: f64 = 1.05;

fn checkout() -> PathBuf {
    PathBuf::from(std::env::var_os("QUIRE_RS_DIR").expect("QUIRE_RS_DIR names a quire-rs checkout"))
}

fn threads() -> String {
    std::env::var("QUIRE_RS_THREADS").unwrap_or_else(|_| "4".into())
}

/// Wall-clock seconds for `command`, whatever its exit status: the suite may
/// carry failing tests, and both modes see the same ones.
fn timed(mut command: Command) -> Duration {
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let start = Instant::now();
    command.status().expect("command starts");
    start.elapsed()
}

fn rounds() -> usize {
    std::env::var("QUIRE_RS_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_ROUNDS)
        .max(DEFAULT_ROUNDS)
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn revision(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git runs");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

#[trace("TC-022", "FR-006-AC-12", "FR-006-CON-1")]
#[test]
#[ignore = "benchmark over a quire-rs checkout; run with QUIRE_RS_DIR set"]
fn tc022_the_runner_adds_under_five_percent_over_quire_rs() {
    let dir = checkout();
    let threads = format!("--test-threads={}", threads());
    let cargo_args = ["--workspace", "--tests"];

    // Compile once, outside the timing, so both modes find a warm cache.
    let status = Command::new("cargo")
        .arg("test")
        .args(cargo_args)
        .arg("--no-run")
        .current_dir(&dir)
        .status()
        .expect("cargo starts");
    assert!(
        status.success(),
        "cargo test --no-run failed in {}",
        dir.display()
    );

    let out_dir = tempfile::tempdir().expect("temp dir");
    let (mut stock, mut runner) = (Vec::new(), Vec::new());
    for round in 0..rounds() {
        let mut stock_cmd = Command::new("cargo");
        stock_cmd
            .arg("test")
            .args(cargo_args)
            .arg("--no-fail-fast")
            .args(["--", &threads])
            .current_dir(&dir);
        let mut runner_cmd = Command::new(env!("CARGO_BIN_EXE_ix-trace-run"));
        runner_cmd
            .arg("--output")
            .arg(out_dir.path().join("result.xml"))
            .args(cargo_args)
            .args(["--", &threads])
            .current_dir(&dir);
        let (s, r) = if round % 2 == 0 {
            let s = timed(stock_cmd);
            (s, timed(runner_cmd))
        } else {
            let r = timed(runner_cmd);
            (timed(stock_cmd), r)
        };
        println!(
            "round {round}: stock {:.2}s runner {:.2}s",
            s.as_secs_f64(),
            r.as_secs_f64()
        );
        stock.push(s.as_secs_f64());
        runner.push(r.as_secs_f64());
    }

    let (stock, runner) = (median(stock), median(runner));
    let ratio = runner / stock;
    println!(
        "TC-022 quire-rs revision {} threads {}: median stock {stock:.2}s runner {runner:.2}s ratio {:.1}%",
        revision(&dir),
        threads,
        ratio * 100.0
    );
    assert!(ratio < LIMIT, "runner costs {:.1}% of stock", ratio * 100.0);
}
