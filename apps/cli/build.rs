#[path = "../../scripts/diagnostic-build.rs"]
mod diagnostic_build;

fn main() {
    diagnostic_build::emit();
}
