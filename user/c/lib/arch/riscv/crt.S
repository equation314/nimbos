.text
.globl _start
_start:
    mv      a0, sp
    and     sp, sp, -16
    tail    __start_main
