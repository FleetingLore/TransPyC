static int global_var = 42;

void TestMacro(void) {
    #define MAX_VALUE 100
    #define PI 3.14159
}

int TestStruct(void) {
    struct Point {
        int x;
        int y;
    };
    struct Point p;
    p.x = 10;
    p.y = 20;
    return (p.x + p.y);
}

int TestPointer(void) {
    int addr = ((void *)4096);
    *((void *)addr) = 42;
    return *(addr);
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

int TestLoop(void) {
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

float TestTypeCast(void) {
    int x = 42;
    float y = ((float)x);
    return y;
}
