use elixir_linter::run;
use std::env::current_dir;

fn main() {
    run(&current_dir().unwrap());
}
