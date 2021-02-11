#![feature(asm)]

#[inline(never)]
#[cfg(target_arch = "x86")]
fn tinycore() {
    let stack = [0u32; 256];
    let rstack = [0u32; 256];
    let mut sp: u32 = 0;
    let mut rp: u32 = 0;
    let mut top: u32 = 0;
    // 由於無法得到以下各指令的位址，此一作法適合用來分析，瞭解 rust 如何配 registers。
    // 可以於完成後反組譯成 .s 檔後加上 labels，再使用 gas 編譯成 .so 檔。
    unsafe {
        asm!(
            // init sp and rp.
            "   mov {sp}, {stack}
                mov {rp}, {rstack}
                ret",
            // sp@
            "   sub {sp}, 4
                mov [{sp}], {top}
                mov {top}, {sp}
                ret",
            // sp!
            "   mov {sp}, {top}
                mov {top}, [{sp}]
                add {sp}, 4
                ret",
            // rp@
            "   sub {sp}, 4
                mov [{rp}], {top}
                mov {top}, {rp}
                ret",
            // rp!
            "   mov {rp}, {top}
                mov {top}, [{sp}]
                add {sp}, 4
                ret",
            // ?dup
            "   or {top}, {top}
                jz 1f",
            // dup
            "   sub {sp}, 4
                mov [{sp}], {top}
            1:
                ret",
            // drop
            "   mov {top}, [{sp}]",
            // nip
            "   add {sp}, 4
                ret",
            // swap
            "   mov {tmp}, [{sp}]
                mov [{sp}], {top}
                mov {top}, {tmp}
                ret",
            // over
            "   sub {sp}, 4
                mov [{sp}], {top}
                mov {top}, 4[{sp}]
                ret",
            // tuck
            "   mov {tmp}, [{sp}]
                sub {sp}, 4
                mov [{sp}], {tmp}
                mov 4[{sp}], {top}
                ret",
            // rot
            "   mov {tmp}, {top}
                mov {top}, 4[{sp}]
                mov {tmp2}, [{sp}]
                mov [{sp}], {tmp}
                mov 4[{sp}], {tmp2}
                ret",
            // -rot
            "   mov {tmp}, {top}
                mov {top}, [{sp}]
                mov {tmp2}, 4[{sp}]
                mov 4[{sp}], {tmp}
                mov [{sp}], {tmp2}
                ret",
            // pick
            "   mov {top}, [{top}*4][{sp}]
                ret",
            // depth
            // TODO: 可能有差一的錯誤
            "   sub {sp}, 4
                mov [{sp}], {top}
                lea {top}, 256[{stack}]
                sub {top}, {sp}
                sar {top}
                sar {top}
                ret",
            // >r
            "   sub {rp}, 4
                mov [{rp}], {top}
                mov {top}, [{sp}]
                add {sp}, 4
                ret",
            // r>
            "   sub {sp}, 4
                mov [{sp}], {top}
                mov {top}, [{rp}]
                add {rp}, 4
                ret",
            // r@
            "   sub {sp}, 4
                mov [{sp}], {top}
                mov {top}, [{rp}]
                ret",
            // rdepth
            // TODO: 可能有差一的錯誤
            "   sub {sp}, 4
                mov [{sp}], {top}
                lea {top}, 256[{rstack}]
                sub {top}, {rp}
                sar {top}
                sar {top}
                ret",
            // rpick
            "   mov {top}, [{top}*4][{rp}]
                ret",
            stack = in(reg) &stack,
            rstack = in(reg) &rstack,
            sp = inout(reg) sp,
            rp = inout(reg) rp,
            top = inout(reg) top,
            tmp = out(reg) _,
            tmp2 = out(reg) _,
        );
    }
}

fn main() {
    tinycore();
}
