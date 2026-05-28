struct Counter {
    int value;
};

void Counter__add(struct Counter* self, int n) {
    self->value += n;
}

int Counter__get(struct Counter* self) {
    return self->value;
}

int main(void) {
    struct Counter c;
    Counter__add(&c, 1);
    Counter__add(&c, 2);
    if (Counter__get(&c) == 3) {
        return 0;
    }
    return 1;
}
