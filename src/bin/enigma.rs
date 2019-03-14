use libenigma::vm;

use getopts::{Options, ParsingStyle};
use std::env;
use std::process;

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();

    let vm = vm::Machine::new();

    // erlexec defaults:
    let args: Vec<String> = vec![
        "/usr/local/Cellar/erlang/21.2.7/lib/erlang/erts-10.2.3/bin/enigma.smp",
        "--",
        "-root",
        "/usr/local/Cellar/erlang/21.2.7/lib/erlang",
        //"/Users/speed/src/otp/release/x86_64-apple-darwin18.0.0",
        //"otp/bootstrap",
        "-progname",
        "enigma",
        "--",
        "-home",
        dirs::home_dir()
            .expect("No home directory")
            .to_str()
            .unwrap(),
        "--",
        "-kernel shell_history enabled",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    vm.preload_modules();

    vm.start(args);

    println!("execution time: {:?}", vm.elapsed_time());
    0
}

fn main() {
    process::exit(run());
}
