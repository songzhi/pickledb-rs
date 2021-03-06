use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::time::Duration;
use std::{thread, time};

mod common;

#[test]
fn auto_dump_poilcy_test() {
    set_test_rsc!("auto_dump_poilcy_test.db");

    // create a DB with AutoDump policy
    let mut db = PickleDb::new("auto_dump_poilcy_test.db", PickleDbDumpPolicy::AutoDump);

    // set a key-value pair
    db.set("key1", &1);

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert_eq!(read_db.get::<i32>("key1").unwrap(), 1);
    }

    // remove a key
    assert!(db.rem("key1"));

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert!(read_db.get::<i32>("key1").is_none());
    }

    // create a list
    db.lcreate("list1");

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert!(read_db.exists("list1"));
        assert_eq!(read_db.llen("list1"), 0);
    }

    // add values to list
    db.lextend("list1", &vec![1,2,3]);

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert_eq!(read_db.llen("list1"), 3);
    }

    // pop an item from a list
    db.lpop::<i32>("list1", 0);

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert_eq!(read_db.llen("list1"), 2);
    }

    // remove an item from a list
    db.lrem_value("list1", &2);

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert_eq!(read_db.llen("list1"), 1);
    }

    // remove a list
    db.lrem_list("list1");

    // verify the change in the DB
    {
        let read_db = PickleDb::load_read_only("auto_dump_poilcy_test.db").unwrap();
        assert!(!read_db.exists("list1"));
    }
}

#[test]
fn read_only_policy_test() {
    set_test_rsc!("read_only_policy_test.db");

    // create a DB and set a value
    let mut db = PickleDb::new("read_only_policy_test.db", PickleDbDumpPolicy::AutoDump);
    db.set("key1", &String::from("value1"));

    // create a read only instance of the same DB
    let mut read_db1 = PickleDb::load_read_only("read_only_policy_test.db").unwrap();

    // set a key-value pair in the read-only DB
    read_db1.set("key2", &String::from("value2"));
    assert!(read_db1.exists("key2"));

    // verify the change isn't dumped to the file
    {
        let read_db2 = PickleDb::load_read_only("read_only_policy_test.db").unwrap();
        assert!(read_db2.exists("key1"));
        assert!(!read_db2.exists("key2"));
    }

    // try to dump data to the file
    read_db1.dump();

    // verify the change isn't dumped to the file
    {
        let read_db2 = PickleDb::load_read_only("read_only_policy_test.db").unwrap();
        assert!(read_db2.exists("key1"));
        assert!(!read_db2.exists("key2"));
    }

    // drop the DB
    drop(read_db1);

    // verify the change isn't dumped to the file
    {
        let read_db2 = PickleDb::load_read_only("read_only_policy_test.db").unwrap();
        assert!(read_db2.exists("key1"));
        assert!(!read_db2.exists("key2"));
    }
}

#[test]
fn dump_upon_request_policy_test() {
    set_test_rsc!("dump_upon_request_policy_test.db");

    // create a DB and set a value
    let mut db = PickleDb::new("dump_upon_request_policy_test.db", PickleDbDumpPolicy::DumpUponRequest);
    db.set("key1", &String::from("value1"));

    // verify file is not yet created
    assert!(PickleDb::load_read_only("dump_upon_request_policy_test.db").is_err());

    // dump to file
    db.dump();

    // verify the change is dumped to the file
    {
        let read_db = PickleDb::load_read_only("dump_upon_request_policy_test.db").unwrap();
        assert!(read_db.exists("key1"));
    }

    // set another key
    db.set("key2", &String::from("value2"));

    // drop DB object
    drop(db);

    // verify the change is dumped to the file
    {
        let read_db = PickleDb::load_read_only("dump_upon_request_policy_test.db").unwrap();
        assert!(read_db.exists("key1"));
        assert!(read_db.exists("key2"));
    }
}

#[test]
fn periodic_dump_policy_test() {
    set_test_rsc!("periodic_dump_policy_test.db");

    // create a DB and set a value
    let mut db = PickleDb::new("periodic_dump_policy_test.db", PickleDbDumpPolicy::PeriodicDump(Duration::new(1, 0)));
    db.set("key1", &String::from("value1"));

    // verify file is not yet created
    assert!(PickleDb::load_read_only("periodic_dump_policy_test.db").is_err());

    // sleep for 0.5 sec
    thread::sleep(time::Duration::from_millis(500));

    // verify file is not yet created
    assert!(PickleDb::load_read_only("periodic_dump_policy_test.db").is_err());

    // sleep for 0.55 sec
    thread::sleep(time::Duration::from_millis(550));

    // make another change in the DB
    db.set("key2", &String::from("value2"));

    // verify the change is dumped to the file
    {
        let read_db = PickleDb::load_read_only("periodic_dump_policy_test.db").unwrap();
        assert!(read_db.exists("key1"));
        assert!(read_db.exists("key2"));
    }

    // make another change in the DB
    db.set("key3", &String::from("value3"));

    // verify the change is not yet dumped to the file
    {
        let read_db = PickleDb::load_read_only("periodic_dump_policy_test.db").unwrap();
        assert!(!read_db.exists("key3"));
    }

    // dumb DB to file
    db.dump();

    // verify the change is now dumped to the file
    {
        let read_db = PickleDb::load_read_only("periodic_dump_policy_test.db").unwrap();
        assert!(read_db.exists("key3"));
    }

    // sleep for 1 more second
    thread::sleep(time::Duration::from_secs(1));

    // make another change in the DB
    db.set("key4", &String::from("value4"));

    // verify the change is dumped to the file
    {
        let read_db = PickleDb::load_read_only("periodic_dump_policy_test.db").unwrap();
        assert!(read_db.exists("key4"));
    }

    // make another change in the DB
    db.set("key5", &String::from("value5"));

    // drop DB and verify change is written to DB
    drop(db);

    // verify the change is dumped to the file
    {
        let read_db = PickleDb::load_read_only("periodic_dump_policy_test.db").unwrap();
        assert!(read_db.exists("key5"));
    }
}