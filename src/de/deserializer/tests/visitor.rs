use {
    super::value::Value,
    serde::de::{self, EnumAccess, SeqAccess, VariantAccess},
    std::fmt::{self, Formatter},
};

pub struct Visitor;

macro_rules! visit_method {
    ( $name:ident () -> $value_type:ident ) => {
        fn $name<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Value::$value_type)
        }
    };
    ( $name:ident ( $base_type:ty ) -> $value_type:ident ) => {
        fn $name<E>(self, value: $base_type) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Value::$value_type(value.into()))
        }
    };
    ( $name:ident ( $self:ident, $deserializer:ident ) -> $forward:ident ) => {
        fn $name<D>($self, $deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            $deserializer.$forward($self)
        }
    };
    ( $name:ident [ $( $base_type:ty => $value_type:ident ),* $(,)* ] ) => {
        fn $name<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::new();

            $(
                let value = Value::$value_type(
                    sequence
                        .next_element::<$base_type>()
                        .unwrap()
                        .unwrap()
                        .into()
                );

                values.push(value);
            )*

            Ok(Value::Sequence(values))
        }
    };
    (
        $name:ident {
            $( $index:expr => $base_type:ty : $value_type:ident ),* $(,)*
        }
    ) => {
        fn $name<A>(self, enum_access: A) -> Result<Self::Value, A::Error>
        where
            A: EnumAccess<'de>,
        {
            let (variant_index, variant_access) =
                enum_access.variant().unwrap();

            let variant_data = match variant_index {
                $(
                    $index => {
                        Value::$value_type(
                            variant_access.newtype_variant().unwrap(),
                        )
                    }
                )*
                _ => Value::Nothing,
            };

            Ok(Value::Enum(variant_index, Box::new(variant_data)))
        }
    };
}

macro_rules! visit_methods {
    ( $( $name:ident $params:tt $( -> $value_type:tt )* ),* $(,)* ) => {
        $(
            visit_method!($name $params $( ->  $value_type )*);
        )*
    };
}

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "unknown")
    }

    visit_methods! {
        visit_bool(bool) -> Bool,
        visit_i8(i8) -> Integer8,
        visit_i16(i16) -> Integer16,
        visit_i32(i32) -> Integer32,
        visit_i64(i64) -> Integer64,
        visit_u8(u8) -> UnsignedInteger8,
        visit_u16(u16) -> UnsignedInteger16,
        visit_u32(u32) -> UnsignedInteger32,
        visit_u64(u64) -> UnsignedInteger64,
        visit_f32(f32) -> Float,
        visit_f64(f64) -> Double,
        visit_char(char) -> Char,
        visit_str(&str) -> String,
        visit_bytes(&[u8]) -> Bytes,
        visit_none() -> None,
        visit_unit() -> Unit,

        visit_some(self, deserializer) -> deserialize_u32,
        visit_newtype_struct(self, deserializer) -> deserialize_i32,

        visit_seq [
            bool => Bool,
            String => String,
            i32 => Integer32,
        ],

        visit_enum {
            0 => bool: Bool,
            1 => String: String,
            2 => i32: Integer32,
        },
    }
}
