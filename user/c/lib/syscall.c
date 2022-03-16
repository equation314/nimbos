#include <stddef.h>
#include <unistd.h>

#include "syscall.h"

ssize_t read(int fd, void* buf, size_t count)
{
    return syscall(SYS_read, fd, buf, count);
}

ssize_t write(int fd, const void* buf, size_t count)
{
    return syscall(SYS_write, fd, buf, count);
}

int getpid(void)
{
    return syscall(SYS_getpid);
}

int sched_yield(void)
{
    return syscall(SYS_yield);
}

void exit(int code)
{
    syscall(SYS_exit, code);
}
