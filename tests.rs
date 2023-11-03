macro_rules! rusqlite_test {
    ($val: expr, $type: ty) => {
        let data = $val;
        let conn =
            rusqlite::Connection::open_in_memory().expect("Unable to open sqlite connection");
        conn.execute(
            "create table if not exists test (id integer primary key, data blob)",
            [],
        )
        .expect("unable to execute");
        conn.execute(
            "insert into test (data) values (?)",
            [dbson::DBson::new(&data)],
        )
        .expect("Unable to insert data");
        let query_data: dbson::DBson<$type> = conn
            .query_row("select data from test", [], |row| row.get(0))
            .expect("Unable to query data");
        let qdata = query_data.into_inner();
        assert!(data == qdata);
    };
}

#[test]
pub fn rusqlite_top_level_test() {
    use std::collections::{BTreeMap, HashMap, HashSet};
    rusqlite_test!(vec![1, 2, 3, 4], Vec<u32>);
    rusqlite_test!(
        vec![1, 2, 3, 4].into_iter().collect::<HashSet<u32>>(),
        HashSet<u32>
    );
    rusqlite_test!(
        vec![("1", "Hello"), ("2", "World"), ("3", "Never"), ("4", "Gonna")]
            .into_iter()
            .map(|(n, w)| (n.to_string(), w.to_string()))
            .collect::<HashMap<String, String>>(),
        HashMap<String, String>
    );
}

#[test]
#[should_panic]
pub fn rusqlite_top_level_test_documents() {
    use std::collections::{BTreeMap, HashMap, HashSet};
    rusqlite_test!(
        vec![(1, "Hello"), (2, "World"), (3, "Never"), (4, "Gonna")]
            .into_iter()
            .map(|(n, w)| (n, w.to_string()))
            .collect::<BTreeMap<u32, String>>(),
        BTreeMap<u32, String>
    );
}

macro_rules! sqlx_test {
    ($val: expr, $type: ty) => {
        let data = $val;
        let mut conn = sqlx::sqlite::SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("Unable to open sqlite connection");
        sqlx::query("create table if not exists test (id integer primary key, data blob)")
            .execute(&mut conn)
            .await
            .expect("unable to execute");
        sqlx::query("insert into test (data) values (?)")
            .bind(dbson::DBson::new(&data))
            .execute(&mut conn)
            .await
            .expect("Unable to insert data");
        let query_data: dbson::DBson<$type> = sqlx::query_scalar("select data from test")
            .fetch_one(&mut conn)
            .await
            .expect("Unable to query data");
        let qdata = query_data.into_inner();
        assert!(data == qdata);
    };
}

#[tokio::test]
pub async fn sqlx_top_level_test() {
    use sqlx::Connection;
    use std::collections::{BTreeMap, HashMap, HashSet};
    sqlx_test!(vec![1, 2, 3, 4], Vec<u32>);
    sqlx_test!(
        vec![1, 2, 3, 4].into_iter().collect::<HashSet<u32>>(),
        HashSet<u32>
    );
    sqlx_test!(
        vec![("1", "Hello"), ("2", "World"), ("3", "Never"), ("4", "Gonna")]
            .into_iter()
            .map(|(n, w)| (n.to_string(), w.to_string()))
            .collect::<HashMap<String, String>>(),
        HashMap<String, String>
    );
}
