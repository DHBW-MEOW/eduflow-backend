use chrono::NaiveDate;

/// enum of all possible values that can be passed to the db
#[derive(Debug)]
pub enum SQLValue {
    Text(String),
    Int32(i32),
    Blob(Vec<u8>),
    Float64(f64),
    Date(NaiveDate),
    Bool(bool),
}

impl Clone for SQLValue {
    fn clone(&self) -> Self {
        match self {
            Self::Text(arg0) => Self::Text(arg0.clone()),
            Self::Int32(arg0) => Self::Int32(*arg0),
            Self::Blob(arg0) => Self::Blob(arg0.clone()),
            Self::Float64(arg0) => Self::Float64(*arg0),
            Self::Date(arg0) => Self::Date(*arg0),
            Self::Bool(arg0) => Self::Bool(*arg0),
        }
    }
}

impl From<String> for SQLValue {
    fn from(val: String) -> Self {
        Self::Text(val)
    }
}
impl From<&str> for SQLValue {
    fn from(val: &str) -> Self {
        Self::Text(val.to_string())
    }
}
impl From<i32> for SQLValue {
    fn from(val: i32) -> Self {
        Self::Int32(val)
    }
}
impl From<Vec<u8>> for SQLValue {
    fn from(val: Vec<u8>) -> Self {
        Self::Blob(val)
    }
}

impl From<NaiveDate> for SQLValue {
    fn from(val: NaiveDate) -> Self {
        Self::Date(val)
    }
}

impl From<bool> for SQLValue {
    fn from(val: bool) -> Self {
        Self::Bool(val)
    }
}

/// macro for creating a parameter map
#[macro_export]
macro_rules! db_param_map {
    ( $( $name:ident : $value:expr ),* $(,)? ) => {
        {
            let mut map: Vec<(String, $crate::db::sql_helper::SQLValue)> = Vec::new();
            $(
                let wrapped = $crate::db::sql_helper::SQLValue::from($value);
                map.push((stringify!($name).to_string(), wrapped));
            )*
            map
        }
    };
}

/// implemented by DBObject
pub trait SQLGenerate {
    /// returns a sql string to create a database table for the struct
    fn get_db_table_create() -> String;
    /// returns a sql string to insert a new row into the database table
    /// parameters are substituted with ?1, ?2, ... ?n
    /// all fields need to be specified, the parameter just ensures that the order can be changed
    fn get_db_insert(fields: Vec<&String>) -> String;
    /// returns a sql string to select rows in a table
    /// where parameters have to be passed into where fields and values will be substituted with ?1, ?2, ... ?n
    fn get_db_select(where_fields: Vec<&String>) -> String;
    /// generates a sql UPDATE statement depending on fields (which will be updated) and where_fields (which will be filtered for)
    fn get_db_update(fields: Vec<&String>, where_fields: Vec<&String>) -> String;
    /// generates a delete statement depending on fields which will be used as where clause
    fn get_db_delete(fields: Vec<&String>) -> String;

    /// returns DBObjIdent, which is unique to a struct (used for local token used_for)
    fn get_db_ident() -> crate::db::DBObjIdent;
    /// converts a rusqlite Row into an object of itself
    fn row_to_struct(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>
    where
        Self: Sized;
}
