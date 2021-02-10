fn p_false(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = 0;
    *top = new_top;
}

fn one(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = 1;
    *top = new_top;
}

fn one_plus(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    stack[*top as usize] += 1;
}

fn two_star(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    stack[*top as usize] *= 2;
}

fn dup(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    let new_top = top.wrapping_sub(1);
    stack[new_top as usize] = stack[*top as usize];
    *top = new_top;
}

fn drop(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    let new_top = top.wrapping_add(1);
    *top = new_top;
}

fn swap(_data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    let next = top.wrapping_add(1);
    let tmp = stack[next as usize];
    stack[next as usize] = stack[*top as usize];
    stack[*top as usize] = tmp;
}

struct Xt(for<'d, 's, 'top> fn(&'d [Xt], &'s mut [isize; 256], &'top mut u8));

#[inline(never)]
fn run(data: &[Xt], stack: &mut [isize; 256], top: &mut u8) {
    for Xt(xt) in data {
        xt(data, stack, top);
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
    run(&data, &mut stack, &mut top);
    println!("{} {} <-", stack[top.wrapping_add(1) as usize], stack[top as usize]);
}
