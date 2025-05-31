#[derive(Debug)]
pub enum SQLWhereValue {
    Text(String),
    Int32(i32),
    Blob(Vec<u8>),
    Float64(f64),
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
            let mut map: Vec<(String, crate::db::sql_helper::SQLWhereValue)> = Vec::new();
            $(
                let wrapped = crate::db::sql_helper::SQLWhereValue::from($value);
                map.push((stringify!($name).to_string(), wrapped));
            )*
            map
        }
    };
}

pub trait SQLGenerate {
    /// returns a sql string to create a database table for the struct
    fn get_db_table_create() -> String;
    /// returns a sql string to insert a new row into the database table
    /// parameters are substituted with ?1, ?2, ... ?n
    fn get_db_insert() -> String;
    /// returns a sql string to select rows in a table
    /// where parameters have to be passed into where fields and values will be substituted with ?1, ?2, ... ?n
    fn get_db_select(where_fields: Vec<&String>) -> String;
    /// generates a sql UPDATE statement depending on fields (which will be updated) and where_fields (which will be filtered for)
    fn get_db_update(fields: Vec<&String>, where_fields: Vec<&String>) -> String;
    /// generates a delete statement depending on fields which will be used as where clause
    fn get_db_delete(fields: Vec<&String>) -> String;

    /// converts a rusqlite Row into an object of itself
    fn row_to_struct(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> where Self: Sized;
}