#include <time.h>

#include "syscall.h"

int nanosleep(const struct timespec *req, struct timespec *rem)
{
    return syscall(SYS_nanosleep, req);
}

int usleep(unsigned useconds)
{
    struct timespec tv = {.tv_sec = useconds / 1000000, .tv_nsec = (useconds % 1000000) * 1000};
    return nanosleep(&tv, &tv);
}

int clock_gettime(clockid_t clk, struct timespec *ts)
{
    return syscall(SYS_clock_gettime, clk, ts);
}

int gettimeofday(struct timeval *restrict tv, void *restrict tz)
{
    struct timespec ts;
    if (!tv) return 0;
    clock_gettime(CLOCK_REALTIME, &ts);
    tv->tv_sec = ts.tv_sec;
    tv->tv_usec = (int)ts.tv_nsec / 1000;
    return 0;
}
