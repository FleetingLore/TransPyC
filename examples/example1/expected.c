#include <stdio.h>
static int global_var = 42;
void TestMacro(void) {
    #define MAX_VALUE 100;
    #define PI 3.14159;
}
void TestStruct(void) {
    int p = Point();
    p.x = 10;
    p.y = 20;
    return (p.x + p.y);
}
void TestPointer(void) {
    int addr = ((void *)4096);
    int value = 42;
    int ptr = addr;
    return 0;
}
int Add(int a, int b) {
    return (a + b);
}
int TestIf(int x) {
    if (x > 0) {
        return (x + 1);
    }
    else
    if (x < 0) {
        return (x - 1);
    }
    else {
        return 0;
    }
}
void TestLoop(void) {
    int sum = 0;
    for (int i = 0; i < 10; i += 1) {
        sum += i;
    }
    return sum;
}
void TestAsm(void) {
    __asm__ volatile (
        "movb $0x0e, %%ah\n\t"
        "    int $0x10\n\t"
        "    : : "a"(c)"
    );
}
void TestTypeCast(void) {
    int x = 42;
    int y = ((float)x);
    return y;
}