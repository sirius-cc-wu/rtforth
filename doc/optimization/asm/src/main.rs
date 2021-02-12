#![feature(asm)]

#[inline(never)]
#[cfg(target_arch = "x86")]
fn tinycore() {
    let stack = [0u32; 256];
    let rstack = [0u32; 256];
    let mut sp: u32 = 0;
    let mut rp: u32 = 0;
    let mut top: u32 = 0;
    let mut tmp: u32 = 0;
    let mut tmp2: u32 = 0;
    // 由於無法得到以下各指令的位址，此一作法適合用來分析，瞭解 rust 如何配 registers。
    // 可以於完成後反組譯成 .s 檔後加上 labels，再使用 gas 編譯成 .so 檔。
    unsafe {
        asm!(
            // init sp and rp.
            "   mov {sp}, {stack}
                mov {rp}, {rstack}
                ret",
            // stackjugglers
            // sp@
            "   sub {sp}, 4
                mov [{sp}], ebx
                mov ebx, {sp}
                ret",
            // sp!
            "   mov {sp}, ebx
                mov ebx, [{sp}]
                add {sp}, 4
                ret",
            // rp@
            "   sub {sp}, 4
                mov [{rp}], ebx
                mov ebx, {rp}
                ret",
            // rp!
            "   mov {rp}, ebx
                mov ebx, [{sp}]
                add {sp}, 4
                ret",
            // ?dup
            "   or ebx, ebx
                jz 1f",
            // dup
            "   sub {sp}, 4
                mov [{sp}], ebx
            1:  ret",
            // drop
            "   mov ebx, [{sp}]",
            // nip
            "   add {sp}, 4
                ret",
            // swap
            "   mov eax, [{sp}]
                mov [{sp}], ebx
                mov ebx, eax
                ret",
            // over
            "   sub {sp}, 4
                mov [{sp}], ebx
                mov ebx, 4[{sp}]
                ret",
            // tuck
            "   mov eax, [{sp}]
                sub {sp}, 4
                mov [{sp}], eax
                mov 4[{sp}], ebx
                ret",
            // rot
            "   mov eax, ebx
                mov ebx, 4[{sp}]
                mov edx, [{sp}]
                mov [{sp}], eax
                mov 4[{sp}], edx
                ret",
            // -rot
            "   mov eax, ebx
                mov ebx, [{sp}]
                mov edx, 4[{sp}]
                mov 4[{sp}], eax
                mov [{sp}], edx
                ret",
            // pick
            "   mov ebx, [ebx*4][{sp}]
                ret",
            // depth
            // TODO: 可能有差一的錯誤
            "   sub {sp}, 4
                mov [{sp}], ebx
                lea ebx, 256[{stack}]
                sub ebx, {sp}
                sar ebx
                sar ebx
                ret",
            // >r
            "   sub {rp}, 4
                mov [{rp}], ebx
                mov ebx, [{sp}]
                add {sp}, 4
                ret",
            // r>
            "   sub {sp}, 4
                mov [{sp}], ebx
                mov ebx, [{rp}]
                add {rp}, 4
                ret",
            // r@
            "   sub {sp}, 4
                mov [{sp}], ebx
                mov ebx, [{rp}]
                ret",
            // rdepth
            // TODO: 可能有差一的錯誤
            "   sub {sp}, 4
                mov [{sp}], ebx
                lea ebx, 256[{rstack}]
                sub ebx, {rp}
                sar ebx
                sar ebx
                ret",
            // rpick
            "   mov ebx, [ebx*4][{rp}]
                ret",
            // calculations
            // +
            "   add ebx, [{sp}]
                add {sp}, 4
                ret",
            // -
            "   sub ebx, [{sp}]
                add {sp}, 4
                ret",
            // 1-
            "   dec ebx
                ret",
            // 1+
            "   inc ebx
                ret",
            // 2-
            "   sub ebx, 2
                ret",
            // 2+
            "   add ebx, 2
                ret",
            // abs
            "   or ebx, ebx
                jns 1f",
            // negate
            "   neg ebx
            1:  ret",
            // u/mod
            "   mov eax, [{sp}]
                xor edx, edx
                div ebx
                mov ebx, eax
                mov [{sp}], edx
                ret",
            // /mod
            "   mov eax, [{sp}]
                cdq
                idiv ebx
                mov ebx, eax
                mov [{sp}], edx
                ret",
            // mod
            "   mov eax, [{sp}]
                cdq
                idiv ebx
                mov ebx, edx
                add {sp}, 4
                ret",
            // /
            "   mov eax, [{sp}]
                cdq
                idiv ebx
                mov ebx, eax
                add {sp}, 4
                ret",
            // *
            "   imul ebx, [{sp}]
                add {sp}, 4
                ret",
            // 2*
            "   shl ebx
                ret",
            // 2/
            "   sar ebx
                ret",
            stack = in(reg) &stack,
            rstack = in(reg) &rstack,
            sp = inout(reg) sp,
            rp = inout(reg) rp,
            inout("ebx") top,
            inout("eax") tmp,
            inout("edx") tmp2,
        );
    }
}

fn main() {
    tinycore();
}
