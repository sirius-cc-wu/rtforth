extern crate rtforth;
extern crate getopts;
extern crate rustyline;

use std::env;
use getopts::Options;
use rtforth::core::VM;
use rtforth::loader::HasLoader;
use rtforth::output::Output;
use rtforth::tools::Tools;
use rtforth::env::Environment;
use rtforth::facility::Facility;
use rtforth::float::Float;
use rtforth::exception::Exception::{
    Bye,
};

#[cfg(not(test))]

#[cfg(not(test))]
fn main() {
    let vm = &mut VM::new(65536);
    let mut bye = false;
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
            match vm.load(&file) {
                None => {},
                Some(e) => {
                    match e {
                        Bye => {},
                        _ => {
                            vm.clear_stacks();
                            vm.reset();
                            println!("{} ", e.description());
                        }
                    }
                    bye = true;
                    break;
                }
            }
        }
        if !bye {
            repl(vm);
        }
    } else {
        print_version();
        println!("Type 'bye' or press Ctrl-D to exit.");
        repl(vm);
    }
}

fn print_version() {
    println!("rtForth v0.1.12, Copyright (C) 2015 Mapacode Inc.");
}

fn repl(vm: &mut VM) {
    let mut rl = rustyline::Editor::new();
    rl.set_response(" ");
    while let Ok(line) = rl.readline("") {
        rl.add_history_entry(&line);
        vm.set_source(&line);
        match vm.evaluate() {
            Some(e) => {
                match e {
                    Bye => break,
                    _ => {
                        vm.clear_stacks();
                        vm.reset();
                        println!("{} ", e.description());
                    }
                }
            },
            None => println!(" ok")
        }
    }
    println!("");
}

#[cfg(not(test))]
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [files] [options]", program);
    print!("{}", opts.usage(&brief));
}
