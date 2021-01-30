fn p_false(_data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    let next_sp = sp.wrapping_sub(1);
    stack[next_sp as usize] = 0;
    *sp = next_sp;
}

fn one(_data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    let next_sp = sp.wrapping_sub(1);
    stack[next_sp as usize] = 1;
    *sp = next_sp;
}

fn one_plus(_data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    stack[*sp as usize] += 1;
}

fn two_star(_data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    stack[*sp as usize] *= 2;
}

fn dup(_data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    let next_sp = sp.wrapping_sub(1);
    stack[next_sp as usize] = stack[*sp as usize];
    *sp = next_sp;
}

struct Xt(for<'d, 's, 'sp> fn(&'d [Xt], &'s mut [isize; 256], &'sp mut u8));

fn run(data: &[Xt], stack: &mut [isize; 256], sp: &mut u8) {
    for Xt(xt) in data {
        xt(data, stack, sp);
    }
}

fn main() {
    let mut stack = [0; 256];
    let mut sp: u8 = 0;
    let mut data: Vec<Xt> = Vec::new();
    data.push(Xt(one));
    data.push(Xt(one_plus));
    data.push(Xt(two_star));
    data.push(Xt(dup));
    run(&data, &mut stack, &mut sp);
    println!("{} {} <-", stack[sp.wrapping_add(1) as usize], stack[sp as usize]);
}
