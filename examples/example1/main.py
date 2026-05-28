import c
import t
import stdio

# 测试宏定义
def TestMacro():
    c.Macro('MAX_VALUE', '100')
    c.Macro('PI', '3.14159')

# 测试类型注解
global_var: t.CStatic | t.CInt = 42

# 测试结构体
def TestStruct():
    class Point:
        def __init__(self):
            self.x: t.CInt = 0
            self.y: t.CInt = 0
    
    p = Point()
    p.x = 10
    p.y = 20
    return p.x + p.y

# 测试指针
def TestPointer():
    addr = c.Memory(0x1000)
    value = 42
    ptr = addr
    *ptr = value
    return *ptr

# 测试函数
def Add(a: t.CInt, b: t.CInt) -> t.CInt:
    return a + b

# 测试条件语句
def TestIf(x: t.CInt) -> t.CInt:
    if x > 0:
        return x + 1
    elif x < 0:
        return x - 1
    else:
        return 0

# 测试循环语句
def TestLoop():
    sum = 0
    for i in range(10):
        sum += i
    return sum

# 测试汇编语句
def TestAsm():
    c.Asm('''
    movb $0x0e, %%ah
    int $0x10
    : : "a"(c)
    ''')

# 测试类型转换
def TestTypeCast():
    x = 42
    y = c.TypeCast('float', x)
    return y
