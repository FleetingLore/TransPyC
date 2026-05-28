use crate::constants::macros::define_consts;

define_consts! {
    /// 帮助信息
    HELP_MESSAGE: &str = r#"""
Usage: python TransPyC.py -f input_file -o output_file [-wh header_files] [-debug debug_file]
       [-cc compile_command] [-cflags compile_flags] [-run] [-args run_args] [-h helper_files]
       -h: Specify helper files (C or Python) to help identify structs, functions, variables, and pointers
"""#;
}
