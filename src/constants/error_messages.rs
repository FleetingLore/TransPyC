use crate::constants::macros::define_map;

define_map! {
    ERROR_MESSAGES,
    "MISSING_ARGS" => "Missing required arguments -f and/or -o",
    "UNKNOWN_ARG" => "Unknown argument",
    "UNSUPPORTED_TYPE" => "Unsupported file type",
    "FAILED_PARSE" => "Failed to parse file",
    "COMPILE_FAILED" => "Compilation or execution failed",
}
