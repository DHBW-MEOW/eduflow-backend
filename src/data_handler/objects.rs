use std::error::Error;

use chrono::NaiveDate;
use db_derive::{DBObject, Selector, SendObject};
use serde::{Deserialize, Serialize};

use crate::{crypt::{crypt_provider::CryptProviders, crypt_types::CryptString, Cryptable}, db::{sql_helper::{SQLGenerate, SQLValue}, DBObjIdent}, db_param_map};

use super::{FromDB, ToDB};

/// create a list of all db object idents here
pub fn get_db_idents() -> [DBObjIdent; 1] {
    [CourseDB::get_db_ident()]
}

// objects
// FIXME: maybe encrypt dates? booleans?

// course: consists of: name (cryptstring)
// topic: consists of: course_id (foreign key), name (cryptstring), details (cryptstring)
// study_goal: consists of: topic_id (foreign key), deadline (date), 
// exam: consists of: course_id (foreign key), name (cryptstring), date (date)

// todo: consists of: name (cryptstring), deadline (date), details (crypstring), completed (bool)

// OBJECTS
// objets have a DB a send and a request type,

// DB types need an id field at first position (i32)
// DB types have an additional user_id field
// DB types derive DBObject

// send types need an id field at first position (Option<i32>)
// send types are used for creating new objects in the db and returning objects to the client, they have to impl CourseSend and FromDB<DBT> with corresponding DB Type
// send types derive Deserialize, Serialize, SendObject

// request types needs a list of parameters which can be filtered on in a select statement
// request types derive Deserialize. Serialize, Selector


// Course
#[derive(Debug, DBObject)]
pub struct CourseDB {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
}

#[derive(Debug, Deserialize, Serialize, SendObject)]
pub struct CourseSend {
    id: Option<i32>,
    name: String,
}

impl ToDB for CourseSend {
    fn to_param_vec(&self, key: &[u8], provider: &CryptProviders) -> Vec<(String, SQLValue)> {
        let name_crypt = CryptString::encrypt(&self.name, key, provider);
        db_param_map! {
            name: SQLValue::Blob(name_crypt.data_crypt)
        }
    }
}

impl FromDB<CourseDB> for CourseSend {
    fn from_dbt(dbt: &CourseDB, key: &[u8], provider: &CryptProviders) -> Result<Self, Box<dyn Error>> {
        let name = dbt.name.decrypt(key, provider);
        Ok(Self { id: Some(dbt.id), name: name? })
    }
}

#[derive(Deserialize, Serialize, Selector)]
pub struct CourseRequest {
    id: Option<i32>,
}

// future db entries
#[derive(Debug, DBObject)]
pub struct Topic {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub details: CryptString,
}
#[derive(Debug, DBObject)]
pub struct StudyGoal {
    pub id: i32,
    pub user_id: i32,

    pub topic_id: i32,
    pub deadline: NaiveDate,
}
#[derive(Debug, DBObject)]
pub struct Exam {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub date: NaiveDate,
}
#[derive(Debug, DBObject)]
pub struct ToDo {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
    pub deadline: NaiveDate,
    pub details: CryptString,
    pub completed: bool,
}

// course: consists of: name (cryptstring)
// topic: consists of: course_id (foreign key), name (cryptstring), details (cryptstring)
// study_goal: consists of: topic_id (foreign key), deadline (date), 
// exam: consists of: course_id (foreign key), name (cryptstring), date (date)

// todo: consists of: name (cryptstring), deadline (date), details (crypstring), completed (bool)