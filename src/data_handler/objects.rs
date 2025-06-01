use std::error::Error;

use chrono::NaiveDate;
use db_derive::{DBObject, Selector, SendObject};
use serde::{Deserialize, Serialize};

use crate::{crypt::{crypt_provider::CryptProviders, crypt_types::CryptString, Cryptable}, db::{sql_helper::{SQLGenerate, SQLValue}, DBObjIdent}, db_param_map};

use super::{FromDB, ToDB};

/// create a list of all db object idents here
pub fn get_db_idents() -> [DBObjIdent; 5] {
    [CourseDB::get_db_ident(), TopicDB::get_db_ident(), StudyGoalDB::get_db_ident(), ExamDB::get_db_ident(), ToDoDB::get_db_ident()]
}

// objects
// FIXME: maybe encrypt dates?

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
#[derive(DBObject)]
pub struct CourseDB {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
}
#[derive(Deserialize, Serialize, Selector)]
pub struct CourseRequest {
    id: Option<i32>,
}
#[derive(Deserialize, Serialize, SendObject)]
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

// Topic
#[derive(DBObject)]
pub struct TopicDB {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub details: CryptString,
}
#[derive(Deserialize, Serialize, Selector)]
pub struct TopicRequest {
    id: Option<i32>,
    course_id: Option<i32>,
}
#[derive(Deserialize, Serialize, SendObject)]
pub struct TopicSend {
    id: Option<i32>,

    course_id: i32,
    name: String,
    details: String,
}
impl ToDB for TopicSend {
    fn to_param_vec(&self, key: &[u8], provider: &CryptProviders) -> Vec<(String, SQLValue)> {
        let name_crypt = CryptString::encrypt(&self.name, key, provider);
        let details_crypt = CryptString::encrypt(&self.details, key, provider);
        db_param_map! {
            course_id: self.course_id,
            name: name_crypt.data_crypt,
            details: details_crypt.data_crypt,
        }
    }
}
impl FromDB<TopicDB> for TopicSend {
    fn from_dbt(dbt: &TopicDB, key: &[u8], provider: &CryptProviders) -> Result<Self, Box<dyn Error>> {
        let name = dbt.name.decrypt(key, provider);
        let details = dbt.details.decrypt(key, provider);
        Ok(Self {
            id: Some(dbt.id),
            course_id: dbt.course_id,
            name: name?,
            details: details?,
        })
    }
}

// Study Goal
#[derive(DBObject)]
pub struct StudyGoalDB {
    pub id: i32,
    pub user_id: i32,

    pub topic_id: i32,
    pub deadline: NaiveDate, // FIXME: encrypt this?
}
#[derive(Deserialize, Serialize, Selector)]
pub struct StudyGoalRequest {
    id: Option<i32>,
    topic_id: Option<i32>,
}
#[derive(Deserialize, Serialize, SendObject)]
pub struct StudyGoalSend {
    id: Option<i32>,

    topic_id: i32,
    deadline: NaiveDate,
}
impl ToDB for StudyGoalSend {
    fn to_param_vec(&self, _: &[u8], _: &CryptProviders) -> Vec<(String, SQLValue)> {
        db_param_map! {
            topic_id: self.topic_id,
            deadline: self.deadline,
        }
    }
}
impl FromDB<StudyGoalDB> for StudyGoalSend {
    fn from_dbt(dbt: &StudyGoalDB, _: &[u8], _: &CryptProviders) -> Result<Self, Box<dyn Error>> {
        Ok(Self { id: Some(dbt.id), topic_id: dbt.topic_id, deadline: dbt.deadline })
    }
}

// Exam
#[derive(DBObject)]
pub struct ExamDB {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub date: NaiveDate, // FIXME: crypt?
}
#[derive(Deserialize, Serialize, Selector)]
pub struct ExamRequest {
    id: Option<i32>,
    course_id: Option<i32>,
}
#[derive(Deserialize, Serialize, SendObject)]
pub struct ExamSend {
    id: Option<i32>,

    course_id: i32,
    name: String,
    date: NaiveDate,
}
impl ToDB for ExamSend {
    fn to_param_vec(&self, key: &[u8], provider: &CryptProviders) -> Vec<(String, SQLValue)> {
        let name_crypt = CryptString::encrypt(&self.name, key, provider);
        db_param_map! {
            course_id: self.course_id,
            name: name_crypt.data_crypt,
            date: self.date,
        }
    }
}
impl FromDB<ExamDB> for ExamSend {
    fn from_dbt(dbt: &ExamDB, key: &[u8], provider: &CryptProviders) -> Result<Self, Box<dyn Error>> {
        let name = dbt.name.decrypt(key, provider);
        Ok(Self {
            id: Some(dbt.id),
            course_id: dbt.course_id,
            name: name?,
            date: dbt.date,
        })
    }
}

// To Do
#[derive(DBObject)]
pub struct ToDoDB {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
    pub deadline: NaiveDate,
    pub details: CryptString,
    pub completed: bool,
}

#[derive(Deserialize, Serialize, Selector)]
pub struct ToDoRequest {
    id: Option<i32>,
    completed: Option<bool>
}
#[derive(Deserialize, Serialize, SendObject)]
pub struct ToDoSend {
    id: Option<i32>,

    name: String,
    deadline: NaiveDate, // FIXME: crypt
    details: String,
    completed: bool,
}
impl ToDB for ToDoSend {
    fn to_param_vec(&self, key: &[u8], provider: &CryptProviders) -> Vec<(String, SQLValue)> {
        let name_crypt = CryptString::encrypt(&self.name, key, provider);
        let details_crypt = CryptString::encrypt(&self.details, key, provider);
        db_param_map! {
            name: name_crypt.data_crypt,
            deadline: self.deadline,
            details: details_crypt.data_crypt,
            completed: self.completed,
        }
    }
}
impl FromDB<ToDoDB> for ToDoSend {
    fn from_dbt(dbt: &ToDoDB, key: &[u8], provider: &CryptProviders) -> Result<Self, Box<dyn Error>> {
        let name = dbt.name.decrypt(key, provider);
        let details = dbt.details.decrypt(key, provider);
        Ok(Self {
            id: Some(dbt.id),
            name: name?,
            deadline: dbt.deadline,
            details: details?,
            completed: dbt.completed,
        })
    }
}