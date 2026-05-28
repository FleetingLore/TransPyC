int main(void) {
    int addr = ((void *)4096);
    int x = 42;
    int p = &x;
    int result = *(p);
    return result;
}
