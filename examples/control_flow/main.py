# if / for / while


def sign(x: int) -> int:
    if x > 0:
        return 1
    elif x < 0:
        return -1
    return 0


def sum_to(n: int) -> int:
    total: int = 0
    for i in range(1, n + 1):
        total += i
    return total


def main() -> int:
    return sign(5) + sum_to(10)
