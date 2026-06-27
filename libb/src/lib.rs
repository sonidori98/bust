use libc::{
    SYS_chdir, SYS_chmod, SYS_exit, SYS_fork, SYS_fstat, SYS_getuid, SYS_link, SYS_lseek,
    SYS_mkdir, SYS_open, SYS_read, SYS_setuid, SYS_stat, SYS_unlink, SYS_wait4, SYS_write, size_t,
    syscall,
};
use std::{arch::naked_asm, ffi::c_void, os::fd::RawFd};

#[unsafe(no_mangle)]
pub extern "sysv64" fn char(string: i64, i: i64) -> i64 {
    let ptr = string as *const u8;
    let value = unsafe { *ptr.offset(i as isize) };
    value as i64
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn chdir(string: i64) -> i64 {
    unsafe { syscall(SYS_chdir, string) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn chmode(string: i64, mode: i64) -> i64 {
    unsafe { syscall(SYS_chmod, string, mode) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn putchar(chr: i64) {
    let bytes = chr.to_ne_bytes();

    let mut len = bytes.len();
    while len > 1 && bytes[len - 1] == 0 {
        len -= 1;
    }

    unsafe {
        syscall(
            SYS_write,
            1 as RawFd,
            bytes.as_ptr() as *const c_void,
            len as size_t,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn printn(n: i64, b: i64) {
    let abs_n = if n < 0 {
        putchar('-' as i64);
        n.unsigned_abs()
    } else {
        n as u64
    };

    printn_unsigned(abs_n, b as u64);
}

fn printn_unsigned(n: u64, b: u64) {
    let a = n / b;
    if a != 0 {
        printn_unsigned(a, b);
    }

    putchar((n % b) as i64 + '0' as i64);
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn ctime(time_vec: *const i64, date: *mut i64) {
    if time_vec.is_null() || date.is_null() {
        return;
    }

    let time: i64 = unsafe { *time_vec };

    let mut days = time / 86400;
    let mut remaining_seconds = time % 86400;
    if remaining_seconds < 0 {
        remaining_seconds += 86400;
        days -= 1;
    }

    let hour = remaining_seconds / 3600;
    let minute = (remaining_seconds % 3600) / 60;
    let second = remaining_seconds % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;

    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };

    const MONTH_STRS: [&[u8; 3]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];

    let month_str = MONTH_STRS[(month - 1) as usize];
    let date_vec = date as *mut u8;

    unsafe {
        *date_vec.offset(0) = month_str[0];
        *date_vec.offset(1) = month_str[1];
        *date_vec.offset(2) = month_str[2];
        *date_vec.offset(3) = b' ';
        *date_vec.offset(4) = (day / 10) as u8 + b'0';
        *date_vec.offset(5) = (day % 10) as u8 + b'0';
        *date_vec.offset(6) = b' ';
        *date_vec.offset(7) = (hour / 10) as u8 + b'0';
        *date_vec.offset(8) = (hour % 10) as u8 + b'0';
        *date_vec.offset(9) = b':';
        *date_vec.offset(10) = (minute / 10) as u8 + b'0';
        *date_vec.offset(11) = (minute % 10) as u8 + b'0';
        *date_vec.offset(12) = b':';
        *date_vec.offset(13) = (second / 10) as u8 + b'0';
        *date_vec.offset(14) = (second % 10) as u8 + b'0';
        *date_vec.offset(15) = 0;
    }
}

#[rustfmt::skip]
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "sysv64" fn printf() {
    naked_asm!(
        "pop r10", // リターンアドレスをどける

        "push r9",
        "push r8",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",

        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        "push r10", // リターンアドレスを戻す

        // [rsp]      : リターンアドレス
        // [rsp + 8]  : r15
        // [rsp + 16] : r14
        // [rsp + 24] : r13
        // [rsp + 32] : r12
        // [rsp + 40] : rbx
        // [rsp + 48] : rdi (fmt)
        // [rsp + 56] : rsi (argv[0])
        // [rsp + 64] : rdx (argv[1])

        "mov r12, [rsp + 48]",  // fmt
        "xor r13, r13",         // index
        "lea r14, [rsp + 56]",  // argv[0]のアドレス

        // fmtから%探し出す
    ".L_loop:",
        "mov al, [r12 + r13]",  // c = fmt[r13]
        "inc r13",
        "cmp al, '%'",
        "je .L_switch",
        "cmp al, 0",
        "je .L_end",

        "movzx rdi, al",
        "call putchar",
        "jmp .L_loop",

    ".L_switch:",
        "mov al, [r12 + r13]",
        "inc r13",

        // %<?>
        "cmp al, 'd'",
        "je .L_case_d_o",
        "cmp al, 'o'",
        "je .L_case_d_o",
        "cmp al, 'c'",
        "je .L_case_c",
        "cmp al, 's'",
        "je .L_case_s",
        "cmp al, '%'",
        "je .L_case_percent",

        // default:
        "movzx rdi, al",
        "call putchar",
        "dec r13",
        "jmp .L_loop",

        // decimal or octal
    ".L_case_d_o:",
        "mov rdi, [r14]",
        "add r14, 8",

        // printn(rdi, c == 'o' ? 8 : 10);
        "mov rsi, 10",
        "cmp al, 'o'",
        "jne 1f",
        "mov rsi, 8",
    "1:",
        "call printn",
        "jmp .L_loop",

        // character
    ".L_case_c:",
        "mov rdi, [r14]",
        "add r14, 8",
        "call putchar",
        "jmp .L_loop",

        // string
    ".L_case_s:",
        "mov r15, [r14]",
        "add r14, 8",
        "xor rbx, rbx",

        // while ((c = char(r15, rbx++)) != '\0')
    ".L_string_loop:",
        "mov al, [r15 + rbx]",  // c = r15[rbx]
        "inc rbx",
        "cmp al, 0",
        "je .L_loop",

        "movzx rdi, al",
        "call putchar",
        "jmp .L_string_loop",

    ".L_case_percent:",
        "mov rdi, '%'",
        "call putchar",
        "jmp .L_loop",

    ".L_end:",
        "pop r10", // リターンアドレスをどける
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "add rsp, 48", // 6 * 8 = 48 bytes
        "push r10", // リターンアドレスを戻す
        "ret",
    );
}

#[rustfmt::skip]
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "sysv64" fn execl() {
    naked_asm!(
        // リターンアドレス rsp
        // スタック引数

        "pop r10",  // リターンアドレスをどける

        "push r9",        // argv[4]
        "push r8",        // argv[3]
        "push rcx",       // argv[2]
        "push rdx",       // argv[1]
        "push rsi",       // argv[0]

        "mov rsi, rsp",

        // argv[0] rsp = rsi
        // argv[1]
        // argv[2]
        // argv[3]
        // argv[4]
        // スタック引数

        "push r10",

        // リターンアドレス  rsp
        // argv[0]  rsi
        // argv[1]
        // argv[2]
        // argv[3]
        // argv[4]
        // スタック引数

        // sys_execve
        "mov rdx, 0",
        "mov rax, 59",
        "syscall",

        // リターンアドレス rsp
        // argv[0]
        // argv[1]
        // argv[2]
        // argv[3]
        // argv[4]
        // スタック引数
        "pop r10",
        "add rsp, 40",
        "push r10",
        "ret"
        // リターンアドレス rsp
        // スタック引数
    );
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn exit() {
    unsafe {
        syscall(SYS_exit, 0);
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn fork() -> i64 {
    unsafe { syscall(SYS_fork) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn fstat(file: i64, status: i64) -> i64 {
    unsafe { syscall(SYS_fstat, file, status) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn getchar() -> i64 {
    let mut c = ' ';
    if unsafe { syscall(SYS_read, 1, &mut c, 1) != 1 } {
        return 0;
    }
    c as i64
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn getuid() -> i64 {
    unsafe { syscall(SYS_getuid) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn gtty(file: i64, ttystat: i64) -> i64 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn lchar(string: i64, i: i64, chr: i64) {
    let ptr = string as *mut u8;

    unsafe {
        let target = ptr.offset(i as isize);
        *target = chr as u8;
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn link(string1: i64, string2: i64) -> i64 {
    unsafe { syscall(SYS_link, string1, string2) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn mkdir(string: i64, mode: i64) -> i64 {
    unsafe { syscall(SYS_mkdir, string, mode) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn open(string: i64, mode: i64) -> i64 {
    unsafe { syscall(SYS_open, string, mode) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn nread(file: i64, buffer: i64, count: i64) -> i64 {
    unsafe { syscall(SYS_read, file, buffer, count) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn seek(file: i64, offset: i64, pointer: i64) -> i64 {
    unsafe { syscall(SYS_lseek, file, offset, pointer) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn setuid(id: i64) -> i64 {
    unsafe { syscall(SYS_setuid, id) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn stat(string: i64, status: i64) -> i64 {
    unsafe { syscall(SYS_stat, string, status) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn stty(file: i64, ttystat: i64) -> i64 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn time(timev: i64) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn unlink(string: i64) -> i64 {
    unsafe { syscall(SYS_unlink, string) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn wait() -> i64 {
    let mut child_status = 0;
    unsafe { syscall(SYS_wait4, -1, &mut child_status, 0) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn nwrite(file: i64, buffer: i64, count: i64) -> i64 {
    unsafe { syscall(SYS_write, file, buffer, count) }
}
