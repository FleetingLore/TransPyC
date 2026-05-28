# c.Memory / c.Addr / c.Cast

import c


def main() -> int:
    addr = c.Memory(0x1000)  # ((void *)0x1000)
    x: int = 42
    p: int = c.Addr(x)  # &x  → int 指针
    result: int = c.Cast(p)  # *(p) → int
    return result
