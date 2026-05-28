use crate::constants::macros::define_consts;

define_consts! {
    /// 默认输入文件名称
    DEFAULT_INPUT_FILE: &str = "test.py";
    /// 默认输出文件名称
    DEFAULT_OUTPUT_FILE: &str = "test.c";
    /// 默认编译命令
    DEFAULT_COMPILE_COMMAND: &str = "gcc";
    /// 默认编译标志
    DEFAULT_COMPILE_FLAGS: &str = "";
}
