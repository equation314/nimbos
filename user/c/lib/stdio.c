#include <stdio.h>
#include <string.h>
#include <unistd.h>

int putchar(int c)
{
    char byte = c;
    return write(stdout, &byte, 1);
}

int puts(const char* s)
{
    int r;
    r = -(write(stdout, s, strlen(s)) < 0 || putchar('\n') < 0);
    return r;
}