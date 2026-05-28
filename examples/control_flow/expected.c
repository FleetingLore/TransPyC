int sign(int x) {
    if (x > 0) {
        return 1;
    }
    else
    if (x < 0) {
        return -1;
    }
    return 0;
}

int sum_to(int n) {
    int total = 0;
    for (int i = 1; i < (n + 1); i += 1) {
        total += i;
    }
    return total;
}

int main(void) {
    return (sign(5) + sum_to(10));
}
