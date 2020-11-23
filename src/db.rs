use core::str::FromStr;

use hyper::Method;
use rusqlite::{params, NO_PARAMS, types::FromSqlError};

use crate::settings;

/// Simple ORM for mocking rules
#[derive(Debug)]
pub struct MockingRule {
    // TODO: Optimize `String` props to `&str` if possible
    pub id: Option<i64>,
    pub request_method: Method,
    pub request_url: String,
    pub request_query: Option<String>,
    pub request_data: Option<String>,
    pub response_status_code: i64,
    pub response_data: Option<String>,
}

impl MockingRule {
    /// Defines name of db table for `MockingRule` model
    const TABLE_NAME: &'static str = "mocking_rules";

    pub fn create_db_table() -> Result<(), String>{
        //! Creates a new DB for model

        let conn = settings::DB.lock().unwrap();
        match conn.execute(
            &format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id                      INTEGER PRIMARY KEY,
                request_method          TEXT NOT NULL,
                request_url             TEXT NOT NULL,
                request_query           TEXT,
                request_data            TEXT,
                response_status_code    INTEGER NOT NULL,
                response_data           TEXT
            )", Self::TABLE_NAME),
            params![],
        ) {
            Ok(_) => return Ok(()),
            Err(e) => panic!(e)
        };
    }

    pub fn create(&mut self) -> Result<(), String> {
        //! Saves instantiated, nonexistent `MockingRule` record.
        if self.id.is_some() {
            return Err("MockingRule already exists. Cannot create records with already existing ID".to_owned());
        }
        
        if Self::find(
            &Some(self.request_url.clone()),
            &self.request_query,
            &Some(self.request_method.clone()),
            &self.request_data
        )?.len() > 0 {
            return Err("Rule on this endpoint already exists!".to_owned());
        }


        let conn = settings::DB.lock().unwrap();
        
        return match conn.execute(
            &format!("
                INSERT INTO {} (
                    request_method,
                    request_url,
                    request_query,
                    request_data,
                    response_status_code,
                    response_data)
                VALUES
                    (?, ?, ?, ?, ?, ?)",
                Self::TABLE_NAME
            ), params![
                self.request_method.as_str(),
                self.request_url,
                self.request_query,
                self.request_data,
                self.response_status_code,
                self.response_data
            ],
        ) {
            Ok(query_result_count) => {
                if query_result_count == 0{
                    return Err("Database failed to perform insert".to_owned());
                }
                self.id = Some(conn.last_insert_rowid());
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    }

    pub fn count_all() -> Result<i64, String> {
        //! Counts total count of all MockingRule records stored in database.
        let conn = settings::DB.lock().unwrap();
        let mut query_statement = conn.prepare(
            &format!("SELECT Count(*) FROM {}", Self::TABLE_NAME)
        ).map_err(|e|e.to_string())?;

        let mut query_result = query_statement.query(NO_PARAMS).map_err(|e|e.to_string())?;
        
        match query_result.next() {
            Ok(row) => {
                return match row {
                    Some(r) => {
                        return match r.get(0) {
                            Ok(val) => Ok(val),
                            Err(e) => Err(e.to_string())
                        };
                    },
                    None => Err("Database Count statement returned no result".to_owned())
                };
            },
            Err(e) => return Err(e.to_string())
        }
    }
    
    pub fn find(
        request_url: &Option<String>,
        request_query: &Option<String>,
        request_method : &Option<Method>,
        request_data: &Option<String>
    ) -> Result<Vec<MockingRule>, String>{
        //! Static function for finding a record in Database.
        // TODO: Make this more configurable!!!

        let conn = settings::DB.lock().unwrap();

        let prepared_request_method : Option<String> = match request_method {
            Some(method) => Some(method.as_str().to_owned()),
            None => None
        };
        
        let mut stmt = conn.prepare(
            &format!(
                "SELECT * FROM {} WHERE request_url {} ? AND request_query {} ? AND request_method {} ? AND request_data {} ?;",
                Self::TABLE_NAME,
                if request_url.is_some(){ "=" } else {"is"},
                if request_query.is_some(){ "=" } else {"is"},
                if prepared_request_method.is_some(){ "=" } else {"is"},
                if request_data.is_some(){ "=" } else {"is"},
            )
        ).map_err(|e|e.to_string())?;
        
        let results = stmt.query_map(
            params![
                request_url,
                request_query,
                prepared_request_method,
                request_data
            ], |row| {

                let id : Option<i64> = row.get(row.column_index("id")?)?;
                let request_method_raw : String = row.get(row.column_index("request_method")?)?;
                let request_method : Method = Method::from_str(&request_method_raw).map_err(
                    |e| FromSqlError::Other(Box::new(e))
                )?;
                
                let request_url : String = row.get(row.column_index("request_url")?)?;
                let request_query : Option<String> = row.get(row.column_index("request_query")?)?;
                let request_data : Option<String> = row.get(row.column_index("request_data")?)?;

                let response_status_code : i64 = row.get(row.column_index("response_status_code")?)?;
                let response_data : Option<String> = row.get(row.column_index("response_data")?)?;

                let output = Ok(MockingRule {
                    id: id,
                    request_method: request_method,
                    request_url: request_url,
                    request_query: request_query,
                    request_data: request_data,
                    response_status_code: response_status_code,
                    response_data: response_data,
                });
                return output;
            }
        ).map_err(|e|e.to_string())?;
        
        let mut output : Vec<MockingRule> = vec![];
        for mocking_rule in results{
            output.push(
                mocking_rule.map_err(|e|e.to_string())?
            );
        }
        
        return Ok(output);
    }
    
    pub fn delete(&mut self) -> Result<(), String> {
        //! Deletes `MockingRule` from a database.
        if self.id.is_none() {
            return Err("Cannot delete MockingRule which does not exist in database.".to_owned());
        }
        
        let conn = settings::DB.lock().unwrap();

        return match conn.execute(
            &format!("DELETE FROM {} WHERE id = ? ;", Self::TABLE_NAME),
            params![self.id],
        ) {
            Ok(query_result_count) => {
                if query_result_count == 0{
                    return Err("Database failed to perform delete".to_owned());
                }
                Ok(())
            },
            Err(e) => Err(e.to_string())
        }
    }
    pub fn display_id(&self) -> String{
        //! Displays `id` as string
        match self.id {
            Some(id) => id.to_string(),
            None => "<None>".to_string()
        }
    }
}

pub fn initialize_db() -> Result<(), String>{
    //! Initializes database.
    MockingRule::create_db_table()?;
    println!("[+] Initializing in-memory sqlite");
    Ok(())
}
