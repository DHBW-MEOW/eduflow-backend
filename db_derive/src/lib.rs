use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, GenericArgument, PathArguments, Type};


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
    // sql string with ?x annotation required for substitution
    let mut parameter_subst_list = "".to_string();

    // populate sql strings (without id)
    fields.named.iter().skip(1).enumerate().for_each(|(i, field)| {
        let type_str = get_sql_type(&field.ty);
        let field_name = field.ident.as_ref().unwrap().to_string();

        db_table.push_str(format!(",{} {}", field_name, type_str).as_str());
        parameter_list.push_str(format!("{field_name},").as_str());
        parameter_subst_list.push_str(format!("?{},", i + 1).as_str());
        //parameter_where.push(value);

    });
    // remove extra comma
    parameter_list.pop();
    parameter_subst_list.pop();

    // rusqlite specific
    // rusqlite row assignment
    let field_assignments = fields.named.iter().enumerate().map(|(i, field)| {
        let field_name = field.ident.as_ref().unwrap();

        quote! {
            #field_name: row.get(#i)?
        }
    });
    

    let generator = quote! {
        // trait definition in main crate
        impl crate::db::sql_helper::SQLGenerate for #struct_name {
            fn get_db_table_create() -> String {
                format!("CREATE TABLE IF NOT EXISTS {} ({})", #struct_name_string, #db_table)
            }

            fn get_db_insert() -> String {
                format!("INSERT INTO {} ({}) VALUES ({})", #struct_name_string, #parameter_list, #parameter_subst_list)
            }

            // TODO: db modify
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

            // rusqlite specific, converts a ruslite row into the struct itself
            fn row_to_struct(row: &rusqlite::Row) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(Self {
                    #(#field_assignments),*
                })
            }

        }
    };

    println!("{generator}");

    generator.into()
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