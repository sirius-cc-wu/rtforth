extern crate rtforth;
extern crate getopts;
extern crate rustyline;
use rtforth::core::VM;
use rtforth::loader::HasLoader;
use rtforth::output::Output;
use rtforth::tools::Tools;
use rtforth::env::Environment;
use rtforth::facility::Facility;
use rtforth::float::Float;
use getopts::Options;
use std::env;

#[cfg(not(test))]
fn main() {
    let vm = &mut VM::new();
    vm.add_output();
    vm.add_tools();
    vm.add_environment();
    vm.add_facility();
    vm.add_float();

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print help menu");
    opts.optflag("v", "version", "print version number");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            panic!(f.to_string());
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
    } else if matches.opt_present("v") {
        print_version();
    } else if !matches.free.is_empty() {
        for file in matches.free {
            vm.load(&file);
        }
        repl(vm);
    } else {
        print_version();
        repl(vm);
    }
}

fn print_version() {
    println!("rtForth v0.1.7, Copyright (C) 2015 Mapacode Inc.");
}

fn repl(vm: &mut VM) {
    let mut rl = rustyline::Editor::new();
    rl.set_response(" ");
    while let Ok(line) = rl.readline("") {
        rl.add_history_entry(&line);
        vm.set_source(&line);
        vm.evaluate();
        println!(" ok");
    }
}

#[cfg(not(test))]
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [files] [options]", program);
    print!("{}", opts.usage(&brief));
}
