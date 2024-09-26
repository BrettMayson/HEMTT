//! A simple way to test SQF with various compilation
//!
//! Requires a running instance of the `arma-bench` server. <https://github.com/brettmayson/arma-bench>
//!
//! I don't know how we want to use this yet, but at least it is easier than launching Arma

use arma_bench::{Client, ServerConfig};
use sqf::display_compare;

mod sqf;

fn main() {
    let client = Client::connect(
        &std::env::var("ARMA_BENCH_HOST").expect("ARMA_BENCH_HOST env var must be set"),
        &ServerConfig::default(),
    )
    .expect("Failed to connect to server");

    let content = r#"
    params ["_a", ["_b", configNull], ["_c", 0, [0]]];
    params [["_cannot", []]];

    x = 180 / 3.1413;
    y = "A" + toUpper "b";

    _a = -5;
    _b = sqrt 2;
    _c = sqrt -1;

    toUpper "🌭";
    "#;

    let res = sqf::compare(&client, content).expect("Failed to compare");
    display_compare(&res);
}
