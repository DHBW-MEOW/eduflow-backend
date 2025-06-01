use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, GenericArgument, PathArguments, Type};

#[proc_macro_derive(Selector)]
pub fn selector_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    //get fields
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data {
        fields
    } else {
        panic!("Selector needs named struct fields");
    };

    // map only Some() fields to (String, SQLWhereValue)
    let select_map = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();

        quote! {
            if self.#field_name.is_some() {
                select_map.push( (stringify!(#field_name).to_string(), crate::db::sql_helper::SQLValue::from(self.#field_name.clone().unwrap())) );
            }
        }

    });

    let generator = quote! {
        impl crate::data_handler::ToSelect for #struct_name {
            fn to_select_param_vec(&self) -> Vec<(String, crate::db::sql_helper::SQLValue)> {
                let mut select_map = vec![];

                #(#select_map)*

                select_map
            }
        }
    };

    generator.into()
}
#[proc_macro_derive(SendObject)]
pub fn send_object_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // get struct name
    let struct_name = input.ident;
    //get fields
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data {
        fields
    } else {
        panic!("SendObject needs named struct fields");
    };

    // first field has to be id
    let id_field_name = fields.named.get(0).expect("SendObject needs at least one field").ident.as_ref().unwrap().to_string();
    if id_field_name != "id" {
        panic!("SendObject first field must be \"id\"!");
    }

    let generator = quote! {
        impl crate::data_handler::Sendable for #struct_name {
            // return id
            fn get_id(&self) -> Option<i32> {
                self.id
            }
        }

    };

    generator.into()
}

#[proc_macro_derive(DBObject)]
pub fn db_object_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // get struct name
    let struct_name = input.ident;
    let struct_name_string = struct_name.to_string();

    // get fields
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = input.data {
        fields
    } else {
        panic!("DBObject needs named struct fields");
    };

    // first field has to be id
    let id_field_name = fields.named.get(0).expect("DBObject needs at least one field").ident.as_ref().unwrap().to_string();
    if id_field_name != "id" {
        panic!("DBObject first field must be \"id\"!");
    }

    // prepare sql strings
    // sql string with field name and data type
    let mut db_table = "id INTEGER PRIMARY KEY AUTOINCREMENT".to_string();
    // sql string with comma seperated list of parameters
    let mut parameter_list = "".to_string();

    // populate sql strings (without id)
    fields.named.iter().skip(1).for_each(|field| {
        let type_str = get_sql_type(&field.ty);
        let field_name = field.ident.as_ref().unwrap().to_string();

        db_table.push_str(format!(",{} {}", field_name, type_str).as_str());
        parameter_list.push_str(format!("{field_name},").as_str());

    });
    // remove extra comma
    parameter_list.pop();

    // rusqlite specific
    // rusqlite row assignment
    let field_assignments = fields.named.iter().enumerate().map(|(i, field)| {
        let field_name = field.ident.as_ref().unwrap();

        quote! {
            #field_name: row.get(#i)?
        }
    });
    

    quote! {
        // trait definition in main crate
        impl crate::db::sql_helper::SQLGenerate for #struct_name {
            fn get_db_table_create() -> String {
                format!("CREATE TABLE IF NOT EXISTS {} ({})", #struct_name_string, #db_table)
            }

            fn get_db_insert(fields: Vec<&String>) -> String {
                let (mut field_names, mut field_subst): (String, String) = fields.iter().enumerate().map(|(i, field)| {
                    (format!("{},", field), format!("?{},", i + 1))
                }).collect();
                // remove trailing ","
                field_names.pop();
                field_subst.pop();

                format!("INSERT INTO {} ({}) VALUES ({})", #struct_name_string, field_names, field_subst)
            }

            // generates a sql select statement with a where statement depending on the where_fields (connected with and)
            fn get_db_select(where_fields: Vec<&String>) -> String {
                // id is excluded in parameter_list
                let mut db_select = format!("SELECT id, {} FROM {}", #parameter_list, #struct_name_string);

                if where_fields.is_empty() {
                    return db_select;
                }

                // we have at least one where condition:
                db_select.push_str(" WHERE");

                where_fields.iter().enumerate().for_each(|(i, field)| {
                    // field + 1 because sql parameters substitution begins at 1 and not 0
                    db_select.push_str(format!(" {} = ?{} AND", field, i + 1).as_str());
                });

                // we added one AND to much, return this instantely
                db_select.strip_suffix(" AND").unwrap().to_string()
            }

            // generates a sql update statement depending on fields (which will be updated) and where_fields (which will be filtered for)
            fn get_db_update(fields: Vec<&String>, where_fields: Vec<&String>) -> String {
                // calculate offset for ? values (we use 1 to fields.len() for fields and fields.len() + 1 till ... for  where fields)
                let where_i_offset = fields.len();

                // map the fields to the SET sql string
                let mut fields: String = fields.iter().enumerate().map(|(i, field)| {
                    format!(" {} = ?{},", field, i + 1)
                }).collect();
                fields.pop();

                // map the where fields to the WHERE sql string
                let where_fields: String = where_fields.iter().enumerate().map(|(i, field)| {
                    format!(" {} = ?{} AND", field, i + 1 + where_i_offset)
                }).collect();
                let where_fields = where_fields.strip_suffix(" AND").unwrap().to_string();

                format!("UPDATE {} SET{} WHERE{}", #struct_name_string, fields, where_fields)
            }

            // generates a sql delete statement depending on fields, which are used for the where clause
            fn get_db_delete(fields: Vec<&String>) -> String {
                // map the where fields to the WHERE sql string
                let fields: String = fields.iter().enumerate().map(|(i, field)| {
                    format!(" {} = ?{} AND", field, i + 1)
                }).collect();
                let fields = fields.strip_suffix(" AND").unwrap().to_string();

                format!("DELETE FROM {} WHERE{}", #struct_name_string, fields)
            }

            fn get_db_ident() -> crate::db::DBObjIdent {
                crate::db::DBObjIdent {
                    db_identifier: #struct_name_string.to_string()
                }
            }

            // rusqlite specific, converts a ruslite row into the struct itself
            fn row_to_struct(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    #(#field_assignments),*
                })
            }

        }
    }.into()
}


fn get_sql_type(field_type: &Type) -> String {
    match field_type {
        Type::Path(type_path) => {

            let mut check_type = field_type;

            let mut result = " NOT NULL".to_string();

            // check for Option<T>
            if type_path.path.segments.len() == 1 {
                let segment = &type_path.path.segments[0];
                if segment.ident == "Option" {
                    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            check_type = inner_type;
                            result = "".into();
                        }
                    }
                }
            }

            let result: String = match check_type {
                Type::Path(inner_path) => {
                    let ident = &inner_path.path.segments.last().unwrap().ident;
                    match ident.to_string().as_str() {
                        "String" => "TEXT".to_string(),
                        "i32" | "i64" => "INTEGER".to_string(),
                        "f64" => "REAL".to_string(),
                        "bool" => "INTEGER".to_string(), // treat booleans as integers in sql
                        "NaiveDate" => "DATE".to_string(),
                        "NaiveDateTime" => "DATETIME".to_string(),
                        _ => "BLOB".to_string()
                    }
                },
                _ => "BLOB".to_string()
            } + &result;

            result

            
        },

        _ => "BLOB".into()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_get_sql_type() {
        let test_cases = vec![
            ("String", "TEXT NOT NULL"),
            ("Option<String>", "TEXT"),
            ("i32", "INTEGER NOT NULL"),
            ("Option<i32>", "INTEGER"),
            ("i64", "INTEGER NOT NULL"),
            ("Option<i64>", "INTEGER"),
            ("f64", "REAL NOT NULL"),
            ("Option<f64>", "REAL"),
            ("Vec<u8>", "BLOB NOT NULL"),
            ("Option<Vec<u8>>", "BLOB"),
            ("bool", "INTEGER NOT NULL"),
            ("Option<bool>", "INTEGER"),
            
            // unknown cases => blob
            ("TestType", "BLOB NOT NULL"),
            ("Option<TestType>", "BLOB"),
        ];

        for (ty_str, expected) in test_cases {
            let ty: Type = parse_str(ty_str).expect("Failed to parse type");
            let sql_type = get_sql_type(&ty);
            assert_eq!(sql_type, expected, "Failed for type {}", ty_str);
        }
    }
}