//! Benchmarks large structured documents through the public widget API.
//!
//! Fixtures default to `benches/structured.json` and `benches/structured.yaml`.
//! Override them with `PROMKIT_STRUCTURED_JSON` and `PROMKIT_STRUCTURED_YAML`.

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

#[path = "structured/common.rs"]
mod common;
#[path = "structured/json.rs"]
mod json;
#[path = "structured/yaml.rs"]
mod yaml;

fn structured(c: &mut Criterion) {
    json::benchmark(c);
    yaml::benchmark(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    targets = structured
}
criterion_main!(benches);
