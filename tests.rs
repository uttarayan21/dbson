macro_rules! rusqlite_test {
    ($val: expr) => {
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
            [dbson::DBson::new(data)],
        )
        .expect("Unable to insert data");
    };
}

#[test]
pub fn rusqlite_top_level_test() {
    rusqlite_test!(vec![1, 2, 3, 4]);
}
