fn p_false(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = 0;
    *top = new_top;
}

fn one(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = 1;
    *top = new_top;
}

fn one_plus(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    stack[*top as usize] += 1;
}

fn two_star(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    stack[*top as usize] *= 2;
}

fn dup(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = stack[*top as usize];
    *top = new_top;
}

fn drop(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    let new_top = top.wrapping_add(1);
    *top = new_top;
}

fn swap(stack: &mut [isize; 256], top: &mut u8, _data: &[Xt]) {
    let next = top.wrapping_add(1);
    let tmp = stack[next as usize];
    stack[next as usize] = stack[*top as usize];
    stack[*top as usize] = tmp;
}

struct Xt(for<'d, 's, 'top> fn(&'s mut [isize; 256], &'top mut u8, &'d [Xt]));

#[inline(never)]
fn run(stack: &mut [isize; 256], top: &mut u8, data: &[Xt]) {
    for Xt(xt) in data {
        xt(stack, top, data);
    }
}

fn main() {
    let mut stack = [0; 256];
    let mut top: u8 = 0;
    let mut data: Vec<Xt> = Vec::new();
    data.push(Xt(one));
    data.push(Xt(one_plus));
    data.push(Xt(two_star));
    data.push(Xt(dup));
    data.push(Xt(drop));
    data.push(Xt(swap));
    run(&mut stack, &mut top, &data);
    println!(
        "{} {} <-",
        stack[top.wrapping_add(1) as usize],
        stack[top as usize]
    );
}
