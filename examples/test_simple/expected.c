#include <stdio.h>
struct Person {
    char* name;
    int age;
};
int add(int a, int b) {
    return (a + b);
}
int main(void) {
    int result = add(5, 3);
    printf("%d\n", result);
    int p = Person("Alice", 30);
    p.greet();
    return 0;
}