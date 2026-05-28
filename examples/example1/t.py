# 类型定义模块

class CType:
    def __init__(self, value=None, *types):
        self.value = value
        self.types = types
    def __merge__(self, types):
        return types
    def __or__(self, other):
        return [self, other]

class CChar(CType):
    def __init__(self, value=None):
        self.CName = 'char'
        super().__init__(value)

class CInt(CType):
    def __init__(self, value=None):
        self.CName = 'int'
        super().__init__(value)

class CShort(CType):
    def __init__(self, value=None):
        self.CName = 'short'
        super().__init__(value)

class CLong(CType):
    def __init__(self, value=None):
        self.CName = 'long'
        super().__init__(value)

class CFloat(CType):
    def __init__(self, value=None):
        self.CName = 'float'
        super().__init__(value)

class CDouble(CType):
    def __init__(self, value=None):
        self.CName = 'double'
        super().__init__(value)

class CVoid(CType):
    def __init__(self, value=None):
        self.CName = 'void'
        super().__init__(value)

class CUnsigned(CType):
    def __init__(self, value=None):
        self.CName = 'unsigned'
        super().__init__(value)

class CUnsignedChar(CType):
    def __init__(self, value=None):
        self.CName = 'unsigned char'
        super().__init__(value)

class CUnsignedInt(CType):
    def __init__(self, value=None):
        self.CName = 'unsigned int'
        super().__init__(value)

class CUnsignedShort(CType):
    def __init__(self, value=None):
        self.CName = 'unsigned short'
        super().__init__(value)

class CUnsignedLong(CType):
    def __init__(self, value=None):
        self.CName = 'unsigned long'
        super().__init__(value)

class CSignedChar(CType):
    def __init__(self, value=None):
        self.CName = 'signed char'
        super().__init__(value)

class CStruct(CType):
    def __init__(self, value=None, name=None):
        self.CName = f'struct {name}' if name else 'struct'
        super().__init__(value)

class CUnion(CType):
    def __init__(self, value=None):
        self.CName = 'union'
        super().__init__(value)

class CEnum(CType):
    def __init__(self, value=None):
        self.CName = 'enum'
        super().__init__(value)

class CTypedef(CType):
    def __init__(self, value=None):
        self.CName = 'typedef'
        super().__init__(value)

class CAuto(CType):
    def __init__(self, value=None):
        self.CName = 'auto'
        super().__init__(value)

class CRegister(CType):
    def __init__(self, value=None):
        self.CName = 'register'
        super().__init__(value)

class CStatic(CType):
    def __init__(self, value=None):
        self.CName = 'static'
        super().__init__(value)

class CExtern(CType):
    def __init__(self, value=None):
        self.CName = 'extern'
        super().__init__(value)

class CConst(CType):
    def __init__(self, value=None):
        self.CName = 'const'
        super().__init__(value)

class CVolatile(CType):
    def __init__(self, value=None):
        self.CName = 'volatile'
        super().__init__(value)

class CSizeT(CType):
    def __init__(self, value=None):
        self.CName = 'size_t'
        super().__init__(value)

class CInt8T(CType):
    def __init__(self, value=None):
        self.CName = 'int8_t'
        super().__init__(value)

class CInt16T(CType):
    def __init__(self, value=None):
        self.CName = 'int16_t'
        super().__init__(value)

class CInt32T(CType):
    def __init__(self, value=None):
        self.CName = 'int32_t'
        super().__init__(value)

class CInt64T(CType):
    def __init__(self, value=None):
        self.CName = 'int64_t'
        super().__init__(value)

class CUInt8T(CType):
    def __init__(self, value=None):
        self.CName = 'uint8_t'
        super().__init__(value)

class CUInt16T(CType):
    def __init__(self, value=None):
        self.CName = 'uint16_t'
        super().__init__(value)

class CUInt32T(CType):
    def __init__(self, value=None):
        self.CName = 'uint32_t'
        super().__init__(value)

class CUInt64T(CType):
    def __init__(self, value=None):
        self.CName = 'uint64_t'
        super().__init__(value)

class CIntPtrT(CType):
    def __init__(self, value=None):
        self.CName = 'intptr_t'
        super().__init__(value)

class CUIntPtrT(CType):
    def __init__(self, value=None):
        self.CName = 'uintptr_t'
        super().__init__(value)

class CPtrDiffT(CType):
    def __init__(self, value=None):
        self.CName = 'ptrdiff_t'
        super().__init__(value)

class CWCharT(CType):
    def __init__(self, value=None):
        self.CName = 'wchar_t'
        super().__init__(value)

class CChar16T(CType):
    def __init__(self, value=None):
        self.CName = 'char16_t'
        super().__init__(value)

class CChar32T(CType):
    def __init__(self, value=None):
        self.CName = 'char32_t'
        super().__init__(value)

class CBool(CType):
    def __init__(self, value=None):
        self.CName = 'bool'
        super().__init__(value)

class CComplex(CType):
    def __init__(self, value=None):
        self.CName = '_Complex'
        super().__init__(value)

class CImaginary(CType):
    def __init__(self, value=None):
        self.CName = '_Imaginary'
        super().__init__(value)

class CPtr(CType):
    def __init__(self, value=None):
        self.CName = '*'
        super().__init__(value)

class CDefine(CType):
    def __init__(self, value=None):
        self.CName = '#define'
        super().__init__(value)

