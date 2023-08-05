extern crate criterion;
extern crate rtforth;

use criterion::{criterion_group, criterion_main, Criterion};
use rtforth::core::Core;
use rtforth::loader::HasLoader;
use rtforth::memory::Memory;
use rtforth::mock_vm::VM;

fn bench_noop(c: &mut Criterion) {
    let vm = &mut VM::new();
    c.bench_function("noop", |b| b.iter(|| vm.noop()));
}

fn bench_find_word_not_exist(c: &mut Criterion) {
    let vm = &mut VM::new();
    c.bench_function("find word not exist", |b| b.iter(|| vm.find("unknown")));
}

fn bench_find_word_at_beginning_of_wordlist(c: &mut Criterion) {
    let vm = &mut VM::new();
    c.bench_function("find word at beginning of worldlist", |b| {
        b.iter(|| vm.find("noop"))
    });
}

fn bench_inner_interpreter_without_nest(c: &mut Criterion) {
    let vm = &mut VM::new();
    let ip = vm.data_space().here();
    let idx = vm.find("noop").expect("noop not exists");
    vm.compile_word(idx);
    vm.compile_word(idx);
    vm.compile_word(idx);
    vm.compile_word(idx);
    vm.compile_word(idx);
    vm.compile_word(idx);
    vm.compile_word(idx);
    c.bench_function("inner interpreter without nest", |b| {
        b.iter(|| {
            vm.state().instruction_pointer = ip;
            vm.run();
        })
    });
}

fn bench_drop(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    c.bench_function("drop", |b| {
        b.iter(|| {
            vm.p_drop();
            vm.s_stack().push(1);
        })
    });
}

fn bench_nip(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(1);
    c.bench_function("nip", |b| {
        b.iter(|| {
            vm.nip();
            vm.s_stack().push(1);
        })
    });
}

fn bench_swap(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    c.bench_function("swap", |b| b.iter(|| vm.swap()));
}

fn bench_dup(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    c.bench_function("dup", |b| {
        b.iter(|| {
            vm.dup();
            vm.s_stack().pop();
        })
    });
}

fn bench_over(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    c.bench_function("over", |b| {
        b.iter(|| {
            vm.over();
            vm.s_stack().pop();
        })
    });
}

fn bench_rot(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    vm.s_stack().push(3);
    c.bench_function("rot", |b| b.iter(|| vm.rot()));
}

fn bench_2drop(c: &mut Criterion) {
    let vm = &mut VM::new();
    c.bench_function("2drop", |b| {
        b.iter(|| {
            vm.s_stack().push(1);
            vm.s_stack().push(2);
            vm.two_drop();
        })
    });
}

fn bench_2dup(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    c.bench_function("2dup", |b| {
        b.iter(|| {
            vm.two_dup();
            vm.two_drop();
        })
    });
}

fn bench_2swap(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    vm.s_stack().push(3);
    vm.s_stack().push(4);
    c.bench_function("2swap", |b| b.iter(|| vm.two_swap()));
}

fn bench_2over(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    vm.s_stack().push(3);
    vm.s_stack().push(4);
    c.bench_function("2over", |b| {
        b.iter(|| {
            vm.two_over();
            vm.two_drop();
        })
    });
}

fn bench_one_plus(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(0);
    c.bench_function("one plus", |b| {
        b.iter(|| {
            vm.one_plus();
        })
    });
}

fn bench_one_minus(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(0);
    c.bench_function("one minus", |b| {
        b.iter(|| {
            vm.one_minus();
        })
    });
}

fn bench_minus(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(0);
    c.bench_function("minus", |b| {
        b.iter(|| {
            vm.dup();
            vm.minus();
        })
    });
}

fn bench_plus(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    c.bench_function("plus", |b| {
        b.iter(|| {
            vm.dup();
            vm.plus();
        })
    });
}

fn bench_star(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    c.bench_function("star", |b| {
        b.iter(|| {
            vm.dup();
            vm.star();
        })
    });
}

fn bench_slash(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    c.bench_function("slash", |b| {
        b.iter(|| {
            vm.dup();
            vm.slash();
        })
    });
}

fn bench_mod(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push(1);
    vm.s_stack().push(2);
    c.bench_function("mod", |b| {
        b.iter(|| {
            vm.p_mod();
            vm.s_stack().push(2);
        })
    });
}

fn bench_slash_mod(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.s_stack().push2(1, 2);
    c.bench_function("slash mod", |b| {
        b.iter(|| {
            vm.slash_mod();
            vm.p_drop();
            vm.s_stack().push(2);
        })
    });
}

/*
    fn bench_compile_words_at_beginning_of_wordlist(c: &mut Criterion) {
        let vm = &mut VM::new();
        c.bench_function("compile words at beginning of wordlist", |b| b.iter(|| {
            vm.set_source("marker empty : main noop noop noop noop noop noop noop noop ; empty");
            vm.evaluate_input();
            vm.s_stack().reset();
        }));
    }

    fn bench_compile_words_at_end_of_wordlist(c: &mut Criterion) {
        let vm = &mut VM::new();
        c.bench_function("compile words at end of wordlist", |b| b.iter(|| {
            vm.set_source("marker empty : main bye bye bye bye bye bye bye bye ; empty");
            vm.evaluate_input();
            vm.s_stack().reset();
        }));
    }

*/

fn bench_to_r_r_fetch_r_from(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.set_source(": main 3 >r r@ drop r> drop ;");
    vm.evaluate_input();
    vm.set_source("' main");
    vm.evaluate_input();
    c.bench_function("to-r r-fetch r-from", |b| {
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
        })
    });
}

fn bench_two_to_r_two_r_fetch_two_r_from(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.set_source(": main 1 2 2>r 2r@ 2drop 2r> 2drop ;");
    vm.evaluate_input();
    vm.set_source("' main");
    vm.evaluate_input();
    c.bench_function("2>r 2r@ 2r<", |b| {
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
        })
    });
}

fn bench_fib(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.set_source(": fib dup 2 < if drop 1 else dup 1- recurse swap 2 - recurse + then ;");
    vm.evaluate_input();
    assert!(vm.last_error().is_none());
    vm.set_source(": main 7 fib drop ;");
    vm.evaluate_input();
    vm.set_source("' main");
    vm.evaluate_input();
    c.bench_function("fib", |b| {
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        })
    });
}

fn bench_repeat(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.set_source(": bench 0 begin over over > while 1 + repeat drop drop ;");
    vm.evaluate_input();
    vm.set_source(": main 8000 bench ;");
    vm.evaluate_input();
    vm.set_source("' main");
    vm.evaluate_input();
    c.bench_function("repeat", |b| {
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        })
    });
}

fn bench_sieve(c: &mut Criterion) {
    let vm = &mut VM::new();
    vm.load_core_fth();
    if vm.last_error().is_some() {
        eprintln!(
            "Error {:?} at {:?}",
            vm.last_error().unwrap(),
            vm.last_token()
        );
    }
    assert_eq!(vm.last_error(), None);
    vm.set_source("CREATE FLAGS 8190 ALLOT   CREATE EFLAG  1 CELLS ALLOT");
    vm.evaluate_input();
    assert_eq!(vm.last_error(), None);
    vm.set_source(
        "
        : PRIMES  ( -- n )  FLAGS 8190 1 FILL  0 3  EFLAG @ FLAGS
            DO   I C@
                IF  DUP I + DUP EFLAG @ <
                    IF    EFLAG @ SWAP
                        DO  0 I C! DUP  +LOOP
                    ELSE  DROP  THEN  SWAP 1+ SWAP
                THEN  2 +
            LOOP  DROP ;
    ",
    );
    vm.evaluate_input();
    assert_eq!(vm.last_error(), None);
    vm.set_source(
        "
        : BENCHMARK  0 1 0 DO  PRIMES NIP  LOOP ;
    ",
    );
    vm.evaluate_input();
    assert_eq!(vm.last_error(), None);
    vm.set_source(
        "
        : MAIN
            FLAGS 8190 + EFLAG !
            BENCHMARK DROP
        ;
    ",
    );
    vm.evaluate_input();
    assert_eq!(vm.last_error(), None);
    vm.set_source("' main");
    vm.evaluate_input();
    c.bench_function("sieve", |b| {
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        })
    });
}

criterion_group!(
    benches,
    bench_noop,
    bench_find_word_not_exist,
    bench_find_word_at_beginning_of_wordlist,
    bench_inner_interpreter_without_nest,
    bench_drop,
    bench_nip,
    bench_swap,
    bench_dup,
    bench_over,
    bench_rot,
    bench_2drop,
    bench_2dup,
    bench_2swap,
    bench_2over,
    bench_one_plus,
    bench_one_minus,
    bench_minus,
    bench_plus,
    bench_star,
    bench_slash,
    bench_mod,
    bench_slash_mod,
    bench_to_r_r_fetch_r_from,
    bench_two_to_r_two_r_fetch_two_r_from,
    bench_fib,
    bench_repeat,
    bench_sieve
);
criterion_main!(benches);
