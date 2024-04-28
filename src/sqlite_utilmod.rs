
#[cfg(test)]
#[path = "./unit_test.rs"]
mod unit_test;

pub mod sqlite_utilmod {
    use serde::{Deserialize, Serialize};
    use rusqlite::{params, Connection, Result, Error};
    use snowflake::SnowflakeIdGenerator;
    use encoding::{Encoding, DecoderTrap};
    use encoding::all::GBK;
    #[derive(Debug,Deserialize,Serialize)]
    pub struct Person{
        pub(crate) id: i64,
        pub(crate) department:String,
        pub(crate) name: String,
        pub(crate) salary: i32,
        pub(crate) avg_salary: Option<f32>,
        pub(crate) create_time: Option<String>,
        pub(crate) update_time: Option<String>,
    }
    #[derive(Debug,Deserialize,Serialize)]
    pub struct PersonResponse{
        pub(crate) id: String,
        pub(crate) department:String,
        pub(crate) name: String,
        pub(crate) salary: i32,
        pub(crate) avg_salary: Option<f32>,
        pub(crate) create_time: Option<String>,
        pub(crate) update_time: Option<String>,
    }
    #[derive(Debug,Deserialize,Serialize)]
    pub struct PersonDTO{
        pub(crate) department:String,
        pub(crate) name: String,
        pub(crate) salary: i32,
    }
    // enum Thing {
    //     Number(u32),
    //     Number16(u16),
    //     String(String),
    // }


   pub fn open_my_db() -> Result<Connection,Error> {
        let path = "./example_db.db3";
        //let env = create_environment_v3_with_os_db_encoding("gbk", "gbk").unwrap();
        let conn = Connection::open(&path)?;
        println!("is_autocommit={}", conn.is_autocommit());
        conn.execute("PRAGMA encoding = \"UTF-8\";", params![])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS person(
                id              BIGINT PRIMARY KEY NOT NULL,
                name            TEXT                NOT NULL,
                department      TEXT                NOT NULL,
                salary           INTEGER            NOT NULL,
                create_time      DATETIME DEFAULT    (datetime('now', 'localtime')),
                update_time      DATETIME DEFAULT    (datetime('now', 'localtime'))
            );",
             params![],
        )?;
        // 查询数据库编码方式
        let encoding: String = conn.query_row("PRAGMA encoding;", [], |row| row.get(0))?;
        println!("Database encoding: {}", encoding);
        
        // 查询表编码方式
        let table_encoding: String = conn.query_row("PRAGMA table_info(person);", [], |row| row.get(2))?;
        println!("Table encoding: {}", table_encoding);
        Ok(conn)
    }

    pub fn insert_person(con:&Connection,p:&PersonDTO) -> Result<usize,Error> {
        let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
        let id = id_generator_generator.generate();
        return Ok(con.execute(
            "insert into person (department, name, salary, id) values (?1, ?2,?3, ?4)",
            params![p.department, p.name, p.salary, id]
        )?);
    }

    pub fn countperson(con:&Connection) -> Result<i64, Error> {
        let res = con.query_row("select count(*) from person;", [], |row| row.get(0)).map_err(|err| {
            // 这里可以更详细地记录错误，比如日志记录，或者更详细的错误信息
            println!("Database query error: {:?}", err);
            err
        })?;
        Ok(res)
    }

    pub fn update_person(con:&Connection, id:i64, p:&PersonDTO) -> Result<usize,Error> {
        let mut sql = String::new();
        sql.push_str("UPDATE person SET ");
        let mut cs = Vec::new();
        let mut param_values: Vec<rusqlite::types::Value> = Vec::new();
        if !(p.department.is_empty()) {
            cs.push("department = ?");
            param_values.push(p.department.clone().into());
        }
        if !(p.name.is_empty()) {
            cs.push("name = ?");
            param_values.push(p.name.clone().into());
        }
        if p.salary >= 0 {
            cs.push("salary = ?");
            param_values.push(p.salary.into());
        }
        sql.push_str(&(cs.join(" and ")));
        sql.push_str(" WHERE id = ? ");
        param_values.push(id.into());
        println!("{}",sql);
        return Ok(con.execute(&sql, rusqlite::params_from_iter(param_values))?);
    }

    pub fn remove_person(con:&Connection, ids: Vec<i64>) -> Result<usize,Error> {
        let mut sql = String::new();
        sql.push_str("DELETE FROM person WHERE id IN ( ");

        for i in 0..ids.len() {
            if i != ids.len()-1 {
                sql.push_str("?,");
            } else {
                sql.push_str("?");
            }
        }
        sql.push_str(" ) ");
        println!("{}",sql);
        return Ok(con.execute(&sql,rusqlite::params_from_iter(ids))?);
    }

    pub fn select_all(con:&Connection, page_no:i32, page_size:i32) -> Result<Vec<PersonResponse>, Error> {
        let limit = page_size ;
        let offset = (page_no-1) * page_size;
        let mut stmt = con.prepare("select id,department,name,salary,create_time,update_time from person ORDER BY create_time DESC LIMIT ? OFFSET ? ;")?;
        let rows = stmt.query_map([&limit, &offset],  |row| {
            let name:String = row.get(2)?;
            let department:String = row.get(1)?;
            let decoded_name = String::from_utf8(name.into_bytes());
            let decoded_department = String::from_utf8(department.into_bytes());
            let id:i64 = row.get(0)?;
            Ok(PersonResponse{
                    id: id.to_string(),
                    department: decoded_department.unwrap(),
                    name: decoded_name.unwrap(),
                    salary: row.get(3)?,
                    avg_salary: None,
                    create_time: row.get(4)?,
                    update_time: row.get(5)?,
                })
        })?;
        let mut res: Vec<PersonResponse> = Vec::new();
        for row in rows {
            let persondata = row?;
            println!("name={}, department={}", persondata.name, persondata.department);
            res.push(PersonResponse{
                id: persondata.id,
                department: persondata.department,
                name: persondata.name,
                salary: persondata.salary,
                avg_salary: None,
                create_time: persondata.create_time,
                update_time: persondata.update_time,
            });
        }
        Ok(res)
    }

    pub fn select_one(con:&Connection, id:i64) -> Result<Option<PersonResponse>, Error> {
        let mut stmt = con.prepare("select id,department,name,salary,create_time,update_time from person WHERE id=:id;")?;
        let mut rows = stmt.query_map(&[(":id", id.to_string().as_str())],  |row| {
            let name:String = row.get(2)?;
            let department:String = row.get(1)?;
            let decoded_name = String::from_utf8(name.into_bytes());
            let decoded_department = String::from_utf8(department.into_bytes());
            let id:i64 = row.get(0)?;
            Ok(PersonResponse{
                    id: id.to_string(),
                    department: decoded_department.unwrap(),
                    name: decoded_name.unwrap(),
                    salary: row.get(3)?,
                    avg_salary: None,
                    create_time: row.get(4)?,
                    update_time: row.get(5)?,
                })
        })?;
        let mut res:Option<PersonResponse> = None;
        for row in rows {
            let persondata = row?;
            println!("name={}, department={}", persondata.name, persondata.department);
            res = Some(persondata);
        }

        Ok(res)
    }


}
