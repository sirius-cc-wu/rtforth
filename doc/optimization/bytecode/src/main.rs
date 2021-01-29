enum ByteCode {
    False,
    One,
    OnePlus,
    TwoStar,
    Dup,
}

fn main() {
    let mut stack = [0; 256];
    let mut sp: u8 = 0;
    let mut data: Vec<ByteCode> = Vec::new();
    data.push(ByteCode::One);
    data.push(ByteCode::OnePlus);
    data.push(ByteCode::TwoStar);
    data.push(ByteCode::Dup);
    for bc in &data {
        match bc {
            ByteCode::False => {
                let next_sp = sp.wrapping_sub(1);
                stack[next_sp as usize] = 0;
                sp = next_sp;
            }
            ByteCode::One => {
                let next_sp = sp.wrapping_sub(1);
                stack[next_sp as usize] = 1;
                sp = next_sp;
            }
            ByteCode::OnePlus => {
                stack[sp as usize] += 1;
            }
            ByteCode::TwoStar => {
                stack[sp as usize] *= 2;
            }
            ByteCode::Dup => {
                let next_sp = sp.wrapping_sub(1);
                stack[next_sp as usize] = stack[sp as usize];
                sp = next_sp;
            }
        }
    }
}
