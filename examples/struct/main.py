# class → struct


class Point:
    x: int
    y: int


def main() -> int:
    p = Point()
    p.x = 3
    p.y = 4
    return p.x + p.y
