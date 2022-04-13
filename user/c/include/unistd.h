#ifndef __UNISTD_H__
#define __UNISTD_H__

#include <stdint.h>

ssize_t read(int, void *, size_t);
ssize_t write(int, const void *, size_t);

pid_t getpid(void);
int sched_yield(void);

pid_t fork(void);
int execve(const char *path);
int wait(int *exitcode);
int waitpid(pid_t pid, int *exitcode);

int usleep(unsigned useconds);
unsigned sleep(unsigned seconds);

#endif // __UNISTD_H__
