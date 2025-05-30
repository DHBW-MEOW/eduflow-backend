#[derive(Debug)]
pub enum SQLWhereValue {
    Text(String),
    Int32(i32),
    Blob(Vec<u8>),
}

impl From<String> for SQLWhereValue {
    fn from(val: String) -> Self {
        Self::Text(val)
    }
}
impl From<&str> for SQLWhereValue {
    fn from(val: &str) -> Self {
        Self::Text(val.to_string())
    }
}
impl From<i32> for SQLWhereValue {
    fn from(val: i32) -> Self {
        Self::Int32(val)
    }
}
impl From<Vec<u8>> for SQLWhereValue {
    fn from(val: Vec<u8>) -> Self {
        Self::Blob(val)
    }
}


#[macro_export]
macro_rules! select_fields {
    ( $( $name:ident : $value:expr ),* $(,)? ) => {
        {
            let mut map: ::std::collections::HashMap<String, db::sql_helper::SQLWhereValue> = ::std::collections::HashMap::new();
            $(
                println!("Got {} = {:?}", stringify!($name), $value);
                let wrapped = db::sql_helper::SQLWhereValue::from($value);
                map.insert(stringify!($name).to_string(), wrapped);
            )*
            map
        }
    };
}
