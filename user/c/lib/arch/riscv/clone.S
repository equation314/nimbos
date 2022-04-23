// __clone(func, arg, stack)
//         a0,   a1,    a2

// syscall(SYS_clone, stack)
//         a7,        a0

.global __clone
.hidden __clone
__clone:
    j   __clone
