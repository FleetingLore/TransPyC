macro_rules! define_map {
    ($name:ident, $($key:expr => $value:expr),* $(,)?) => {
        pub const $name: &[(&str, &str)] = &[
            $(($key, $value)),*
        ];
    };
}

macro_rules! define_consts {
    ($($(#[$attr:meta])* $name:ident: $ty:ty = $value:expr;)*) => {
        $($(#[$attr])* pub const $name: $ty = $value;)*
    };
}

pub(crate) use define_consts;
pub(crate) use define_map;
