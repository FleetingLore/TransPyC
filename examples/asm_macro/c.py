# C语法定义模块

class Asm:
    def __init__(self, code, *args):
        self.code = code
        self.args = args

class State:
    def __init__(self):
        pass

class ClassPoint:
    def __init__(self):
        pass

class ClassList:
    def __init__(self):
        self.Items = []
    def Append(self, item):
        self.Items.append(item)

class Memory:
    def __init__(self, addr):
        self.addr = addr

class Dereference:
    def __init__(self, ptr):
        self.ptr = ptr

class Reference:
    def __init__(self, var):
        self.var = var

class Macro:
    def __init__(self, name, value):
        self.name = name
        self.value = value

class Ast:
    def __init__(self):
        pass

class TypeCast:
    def __init__(self, type_name, value):
        self.type_name = type_name
        self.value = value

class Esp:
    def __init__(self):
        pass

class Ebp:
    def __init__(self):
        pass

class Addr:
    def __init__(self, addr):
        self.addr = addr

class Ptr:
    def __init__(self, addr, value=None, type=None):
        self.addr = addr
        self.value = value
        self.type = type
