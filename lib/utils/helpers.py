# 工具函数

import os
import subprocess
from lib.constants.config import ERROR_MESSAGES


def detect_file_type(file_path):
    """检测文件类型"""
    _, ext = os.path.splitext(file_path)
    return ext.lower()


def execute_command(command, shell=True, capture_output=True, text=True):
    """执行命令"""
    try:
        result = subprocess.run(
            command,
            shell=shell,
            check=True,
            capture_output=capture_output,
            text=text
        )
        return result
    except subprocess.CalledProcessError as e:
        print(f'{ERROR_MESSAGES["COMPILE_FAILED"]}: {e}')
        if e.stderr:
            print(f'Error output: {e.stderr}')
        return None


def get_file_content(file_path):
    """获取文件内容"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return f.read()
    except Exception as e:
        print(f'{ERROR_MESSAGES["FAILED_PARSE"]} {file_path}: {e}')
        return None


def write_file_content(file_path, content):
    """写入文件内容"""
    try:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    except Exception as e:
        print(f'Failed to write file {file_path}: {e}')
        return False


def append_file_content(file_path, content):
    """追加文件内容"""
    try:
        with open(file_path, 'a', encoding='utf-8') as f:
            f.write(content)
            f.write('\n')
        return True
    except Exception as e:
        print(f'Failed to append to file {file_path}: {e}')
        return False


def is_standard_library(module_name):
    """检查是否是标准库"""
    standard_libs = ['stdio', 'stdlib', 'string', 'math', 'time', 'sys', 'os']
    return module_name in standard_libs


def extract_array_size(type_name):
    """提取数组大小"""
    array_sizes = []
    base_type = type_name
    
    while '[' in base_type:
        size_start = base_type.rfind('[')
        size_end = base_type.rfind(']')
        if size_start != -1 and size_end != -1:
            array_size = base_type[size_start+1:size_end]
            array_sizes.append(array_size)
            base_type = base_type[:size_start]
    
    array_size_str = ''
    for size in reversed(array_sizes):
        array_size_str += f'[{size}]'
    
    return base_type, array_size_str


def check_storage_class(type_name):
    """检查存储类修饰符"""
    storage_class = ''
    type_part = type_name
    
    if type_name.startswith('static ') or type_name.startswith('extern '):
        storage_parts = type_name.split(' ', 1)
        storage_class = storage_parts[0]
        type_part = storage_parts[1]
    
    return storage_class, type_part


def build_array_initialization(elements, use_single_quote=False):
    """构建数组初始化代码"""
    elements_str = ', '.join(elements)
    return f'{{ {elements_str} }}'


def format_error_message(error_type, *args):
    """格式化错误消息"""
    if error_type in ERROR_MESSAGES:
        message = ERROR_MESSAGES[error_type]
        if args:
            return message.format(*args)
        return message
    return f'Unknown error: {error_type}'


def validate_args(args):
    """验证命令行参数"""
    if 'Input' not in args or 'Output' not in args:
        return False, ERROR_MESSAGES['MISSING_ARGS']
    return True, None


def get_indentation(level=1):
    """获取缩进"""
    return '    ' * level
