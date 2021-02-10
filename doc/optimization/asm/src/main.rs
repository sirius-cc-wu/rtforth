#![feature(asm)]

#[inline(never)]
#[cfg(target_arch = "x86")]
fn tinycore() {
    // reg 0
    let mut sp: u32 = 0;
    // reg 1
    let mut top: u32 = 0;
    // reg 2
    let mut tmp: u32 = 0;
    // reg 3
    let stack = [0u32; 256];
    unsafe {
        asm!(
            // init sp
            "mov {0}, {3}",
            // dup
            "sub {0}, 4",
            "mov [{0}], {1}",
            "ret",
            // drop
            "mov {1}, [{0}]",
            "add {0}, 4",
            "ret",
            // nip
            "add {0}, 4",
            "ret",
            // swap
            "mov {2}, [{0}]",
            "mov [{0}], {1}",
            "mov {1}, {2}",
            "ret",
            // over
            "sub {0}, 4",
            "mov [{0}], {1}",
            "mov {1}, 4[{0}]",
            "ret",
            // tuck
            "mov {2}, [{0}]",
            "sub {0}, 4",
            "mov [{0}], {2}",
            "mov 4[{0}], {1}",
            "ret",
            inout(reg) sp,
            inout(reg) top,
            inout(reg) tmp,
            in(reg) &stack,
        );
    }
}

fn main() {
    tinycore();
}
