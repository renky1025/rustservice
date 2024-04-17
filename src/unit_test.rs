
pub fn greeting(name: &str) -> String {
    format!("Hello {}!", name)
}

#[cfg(test)]
mod unit_test {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    #[ignore]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
    use super::*;

    #[test]
    fn greeting_contains_name() {
        let result = greeting("Carol");
        assert!(result.contains("Carol"));
    }
    use crate::sqlite_utilmod::sqlite_utilmod::open_my_db;
    use crate::sqlite_utilmod::sqlite_utilmod::select_all;
    use crate::sqlite_utilmod::sqlite_utilmod::PersonDTO;
    use crate::sqlite_utilmod::sqlite_utilmod::insert_person;
    #[test]
    fn insert_fun() {
        let con = open_my_db();
        match con {
            Ok(conn) =>{
                let person: PersonDTO = PersonDTO {
                    department: String::from("工程部"),
                    name: String::from("无敌老六"),
                    salary: 100000,
                };
                let r = insert_person(&conn, &person);
                println!("{}", r.unwrap());

                let res = select_all(&conn, 1, 10);
                println!("{:?}", res.unwrap());
            },
            Err(err) =>
            println!("Error: {:?}", err),
        }

    }
}