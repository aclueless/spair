pub enum Value {
    None,
    Default,
    Bool(bool),
    Char(char),
    Isize(isize),
    Usize(usize),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    F32(f32),
    F64(f64),
    String(String),
}

pub trait ValueChanged: Copy {
    fn check_value_changed(self, value: &mut Value) -> bool;
}

macro_rules! impl_value_changed {
    ($($Variant:ident $Type:ty)+) => {
        $(
        impl ValueChanged for $Type {
            fn check_value_changed(self, value: &mut Value) -> bool {
                if let Value::$Variant(old_value) = value {
                    if *old_value != self {
                        *old_value = self;
                        return true;
                    }
                }
                else{
                    *value = Value::$Variant(self);
                    return true;
                }
                false
            }
        }
        )+
    };
}

impl_value_changed! {
    Bool bool
    Char char
    Isize isize
    Usize usize
    I8 i8
    U8 u8
    I16 i16
    U16 u16
    I32 i32
    U32 u32
    I64 i64
    U64 u64
    I128 i128
    U128 u128
    F32 f32
    F64 f64
}
