use core::arch::asm;

const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_GETPID: usize = 39;
const SYSCALL_CLONE: usize = 56;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_WAITPID: usize = 61;
const SYSCALL_GET_TIME: usize = 96;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let ret;
    unsafe {
        asm!(
            "svc #0",
            inlateout("x0") args[0] => ret,
            in("x1") args[1],
            in("x2") args[2],
            in("x8") id,
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_clone(entry: fn(usize) -> i32, arg: usize, newsp: usize) -> usize {
    let ret;
    unsafe {
        asm!("
            // align stack and save entry,arg to the new stack
	        and x2, x2, #-16
            stp x0, x1, [x2, #-16]!

            // syscall(SYSCALL_CLONE, newsp)
            mov x0, x2
            mov x8, {sys_clone}
            svc #0

            cbz x0, 1f
            // parent
            ret
        1:
            // child
            ldp x1, x0, [sp], #16
            blr x1
            // syscall(SYSCALL_EXIT, ret)
            mov x8, {sys_exit}
            svc #0",
            sys_clone = const SYSCALL_CLONE,
            sys_exit = const SYSCALL_EXIT,
            inlateout("x0") entry => ret,
            in("x1") arg,
            in("x2") newsp,
        );
    }
    ret
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}
