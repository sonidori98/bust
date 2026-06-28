use libc::{
    SYS_chdir, SYS_chmod, SYS_chown, SYS_close, SYS_creat, SYS_execve, SYS_exit, SYS_fork,
    SYS_fstat, SYS_getuid, SYS_link, SYS_lseek, SYS_mkdir, SYS_open, SYS_read, SYS_setuid,
    SYS_stat, SYS_time, SYS_unlink, SYS_wait4, SYS_write, size_t, syscall,
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
pub extern "sysv64" fn chmod(string: i64, mode: i64) -> i64 {
    unsafe { syscall(SYS_chmod, string, mode) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn chown(string: i64, owner: i64) -> i64 {
    unsafe { syscall(SYS_chown, string, owner) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn close(file: i64) -> i64 {
    unsafe { syscall(SYS_close, file) }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn creat(string: i64, mode: i64) -> i64 {
    unsafe { syscall(SYS_creat, string, mode) }
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
pub extern "sysv64" fn execv(string: i64, argv: i64, count: i64) {
    let mut args: Vec<i64> = Vec::with_capacity(count as usize + 1);
    for i in 0..count {
        args.push(unsafe { *(argv as *const i64).offset(i as isize) });
    }
    args.push(0);
    let envp: i64 = 0;
    unsafe {
        syscall(
            SYS_execve,
            string,
            args.as_ptr() as i64,
            &envp as *const i64 as i64,
        );
    }
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
    let mut c: u8 = 0;
    if unsafe { syscall(SYS_read, 0 as RawFd, &mut c as *mut u8 as *mut c_void, 1) != 1 } {
        return 0;
    }
    c as i64
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn getuid() -> i64 {
    unsafe { syscall(SYS_getuid) }
}

struct Sgttyb {
    ispeed: u8,
    ospeed: u8,
    erase: u8,
    kill: u8,
    flags: i32,
}

const SG_LCASE: i32 = 0o0000001;
const SG_ECHO: i32 = 0o0000010;
const SG_CBREAK: i32 = 0o0000020;
const SG_RAW: i32 = 0o0000040;
const SG_CRMOD: i32 = 0o0000200;
const SG_NL2: i32 = 0o0000400;
const SG_TANDEM: i32 = 0o0001000;
const SG_XTABS: i32 = 0o040000;

#[unsafe(no_mangle)]
pub extern "sysv64" fn gtty(file: i64, ttystat: i64) -> i64 {
    let mut tios: libc::termios = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::tcgetattr(file as i32, &mut tios) };
    if ret < 0 {
        return -1;
    }
    let mut flags: i32 = 0;

    if tios.c_lflag & libc::ECHO as u32 != 0 {
        flags |= SG_ECHO;
    }
    let is_raw = tios.c_iflag
        & (libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON) as u32
        == 0
        && tios.c_oflag & libc::OPOST as u32 == 0
        && tios.c_cflag & (libc::CSIZE as u32) == libc::CS8 as u32
        && !(tios.c_lflag & (libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN) as u32 != 0);
    let is_cbreak =
        !(tios.c_lflag & libc::ICANON as u32 != 0) && !(tios.c_lflag & libc::ECHO as u32 != 0);
    if is_raw {
        flags |= SG_RAW;
    } else if is_cbreak {
        flags |= SG_CBREAK;
    }
    if tios.c_iflag & libc::ICRNL as u32 != 0 && tios.c_oflag & libc::ONLCR as u32 != 0 {
        flags |= SG_CRMOD;
    }
    if tios.c_iflag & libc::IUCLC as u32 != 0 {
        flags |= SG_LCASE;
    }
    if tios.c_iflag & libc::IXOFF as u32 != 0 {
        flags |= SG_TANDEM;
    }
    if tios.c_oflag & libc::OPOST as u32 != 0 && tios.c_oflag & libc::ONLCR as u32 != 0 {
        flags |= SG_NL2;
    }
    if tios.c_oflag & libc::TABDLY as u32 == libc::TAB3 as u32 {
        flags |= SG_XTABS;
    }

    let sg = Sgttyb {
        ispeed: unsafe { libc::cfgetispeed(&tios) } as u8,
        ospeed: unsafe { libc::cfgetospeed(&tios) } as u8,
        erase: tios.c_cc[libc::VERASE] as u8,
        kill: tios.c_cc[libc::VKILL] as u8,
        flags,
    };
    unsafe {
        std::ptr::write_unaligned(ttystat as *mut Sgttyb, sg);
    }
    0
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
    let sg = unsafe { std::ptr::read_unaligned(ttystat as *const Sgttyb) };
    let mut tios: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(file as i32, &mut tios) } < 0 {
        return -1;
    }

    if sg.flags & SG_RAW != 0 {
        tios.c_iflag &=
            !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON) as u32;
        tios.c_oflag &= !libc::OPOST as u32;
        tios.c_cflag = (tios.c_cflag & !libc::CSIZE as u32) | libc::CS8 as u32;
        tios.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN) as u32;
        tios.c_cc[libc::VMIN] = 1;
        tios.c_cc[libc::VTIME] = 0;
    } else if sg.flags & SG_CBREAK != 0 {
        tios.c_lflag &= !(libc::ICANON | libc::ECHO) as u32;
    } else {
        if sg.flags & SG_ECHO != 0 {
            tios.c_lflag |= libc::ECHO as u32;
        } else {
            tios.c_lflag &= !libc::ECHO as u32;
        }
    }

    if sg.flags & SG_CRMOD != 0 {
        tios.c_iflag |= libc::ICRNL as u32;
        tios.c_oflag |= libc::ONLCR as u32;
    } else {
        tios.c_iflag &= !libc::ICRNL as u32;
        tios.c_oflag &= !libc::ONLCR as u32;
    }

    if sg.flags & SG_LCASE != 0 {
        tios.c_iflag |= libc::IUCLC as u32;
        tios.c_oflag |= libc::OLCUC as u32;
    } else {
        tios.c_iflag &= !libc::IUCLC as u32;
        tios.c_oflag &= !libc::OLCUC as u32;
    }

    if sg.flags & SG_TANDEM != 0 {
        tios.c_iflag |= libc::IXOFF as u32;
    } else {
        tios.c_iflag &= !libc::IXOFF as u32;
    }

    if sg.flags & SG_NL2 != 0 {
        tios.c_oflag |= libc::ONLCR as u32;
    }

    if sg.flags & SG_XTABS != 0 {
        tios.c_oflag = (tios.c_oflag & !libc::TABDLY as u32) | libc::TAB3 as u32;
    } else {
        tios.c_oflag &= !libc::TABDLY as u32;
    }

    if sg.ispeed != 0 {
        unsafe { libc::cfsetispeed(&mut tios, sg.ispeed as u32) };
    }
    if sg.ospeed != 0 {
        unsafe { libc::cfsetospeed(&mut tios, sg.ospeed as u32) };
    }
    tios.c_cc[libc::VERASE] = sg.erase as u8;
    tios.c_cc[libc::VKILL] = sg.kill as u8;

    unsafe { libc::tcsetattr(file as i32, libc::TCSANOW, &tios) as i64 }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn time(timev: i64) {
    let tv = unsafe { syscall(SYS_time, std::ptr::null::<i64>()) };
    unsafe {
        *(timev as *mut i64) = tv;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtty_invalid_fd() {
        let mut buf = [0i64; 8];
        assert_eq!(gtty(-1, &mut buf as *mut _ as i64), -1);
    }

    #[test]
    fn test_stty_invalid_fd() {
        let buf = [0i64; 8];
        assert_eq!(stty(-1, &buf as *const _ as i64), -1);
    }

    #[test]
    fn test_gtty_pipe_fd() {
        let mut fds = [0i32; 2];
        let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
        assert_eq!(ret, 0);
        let mut buf = [0i64; 8];
        assert_eq!(gtty(fds[0] as i64, &mut buf as *mut _ as i64), -1);
        assert_eq!(gtty(fds[1] as i64, &mut buf as *mut _ as i64), -1);
        unsafe { libc::close(fds[0]); libc::close(fds[1]); }
    }

    #[test]
    fn test_stty_pipe_fd() {
        let mut fds = [0i32; 2];
        let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
        assert_eq!(ret, 0);
        let buf = [0i64; 8];
        assert_eq!(stty(fds[0] as i64, &buf as *const _ as i64), -1);
        assert_eq!(stty(fds[1] as i64, &buf as *const _ as i64), -1);
        unsafe { libc::close(fds[0]); libc::close(fds[1]); }
    }

    #[test]
    fn test_gtty_stty_flag_roundtrip_on_tty() {
        let fd = unsafe { libc::open("/dev/tty\0".as_ptr() as *const i8, libc::O_RDWR) };
        if fd < 0 {
            let fd2 = unsafe { libc::open("/dev/console\0".as_ptr() as *const i8, libc::O_RDWR) };
            if fd2 < 0 {
                eprintln!("skipping tty test: no tty available");
                return;
            }
            let mut buf = [0i64; 8];
            assert_eq!(gtty(fd2 as i64, &mut buf as *mut _ as i64), 0);
            assert_eq!(stty(fd2 as i64, &buf as *const _ as i64), 0);
            unsafe { libc::close(fd2); }
        } else {
            let mut buf = [0i64; 8];
            assert_eq!(gtty(fd as i64, &mut buf as *mut _ as i64), 0);
            assert_eq!(stty(fd as i64, &buf as *const _ as i64), 0);
            unsafe { libc::close(fd); }
        }
    }

    #[test]
    fn test_sgttyb_flag_values() {
        assert_eq!(SG_LCASE, 0o0000001);
        assert_eq!(SG_ECHO, 0o0000010);
        assert_eq!(SG_CBREAK, 0o0000020);
        assert_eq!(SG_RAW, 0o0000040);
        assert_eq!(SG_CRMOD, 0o0000200);
        assert_eq!(SG_NL2, 0o0000400);
        assert_eq!(SG_TANDEM, 0o0001000);
        assert_eq!(SG_XTABS, 0o040000);
    }

    #[test]
    fn test_sgttyb_struct_size() {
        assert_eq!(std::mem::size_of::<Sgttyb>(), 8);
    }
}
