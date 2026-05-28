# 综合示例: struct + method + if


class Counter:
    value: int

    def add(self, n: int) -> None:
        self.value += n

    def get(self) -> int:
        return self.value


def main() -> int:
    c = Counter()
    c.add(1)
    c.add(2)
    if c.get() == 3:
        return 0
    return 1
