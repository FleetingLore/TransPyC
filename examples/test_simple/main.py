# 简单综合测试
import c
import stdio  # std: standard
import t


def add(a: int, b: int) -> int:
    return a + b


class Person:
    name: str
    age: int

    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age

    def greet(self):
        print("Hello")


def main() -> int:
    result = add(5, 3)
    print(result)

    p = Person("Alice", 30)
    p.greet()

    return 0
