#include <stdint.h>
#include <unistd.h>

#include "syscall.h"

ssize_t read(int fd, void *buf, size_t count)
{
    return syscall(SYS_read, fd, buf, count);
}

ssize_t write(int fd, const void *buf, size_t count)
{
    return syscall(SYS_write, fd, buf, count);
}

pid_t getpid(void)
{
    return syscall(SYS_getpid);
}

int sched_yield(void)
{
    return syscall(SYS_yield);
}

_Noreturn void exit(int code)
{
    for (;;) syscall(SYS_exit, code);
}

pid_t fork(void)
{
    return syscall(SYS_fork);
}

int execve(const char *path)
{
    return syscall(SYS_exec, path);
}

int waitpid(pid_t pid, int *exit_code)
{
    for (;;) {
        int ret = syscall(SYS_waitpid, pid, exit_code);
        if (ret == -2) {
            sched_yield();
        } else {
            return ret;
        }
    }
}

int wait(int *exit_code)
{
    return waitpid(-1, exit_code);
}
