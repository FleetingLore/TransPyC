# c.Asm / c.Macro

import c


def main() -> int:
    c.Asm("nop")
    c.Macro("SIZE", "256")
    return 0
