use std::sync::Once;
use std::time;
use std::thread;
use std::io::Read;
use keyval_store::lib_main;
use actix_web::rt::System;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        thread::spawn(move ||{
            let sys = System::new();
            sys.block_on(lib_main(8081)).unwrap();
        });
        thread::sleep(time::Duration::from_secs(5));
    });
}

fn get_as_string(query: String) -> String {
    let mut res = reqwest::blocking::get(query).unwrap();
    let mut data_out = String::new();
    res.read_to_string(&mut data_out).unwrap();
    data_out
}

#[test]
fn test_example() {
    setup();

    let data_in = "456";
    let my_key = "http://0.0.0.0:8081/v1/test_example";
    let _res = reqwest::blocking::get(format!("{my_key}/set/{data_in}")).unwrap();
    let mut res = reqwest::blocking::get(format!("{my_key}/get")).unwrap();
    let mut data_out = String::new();
    res.read_to_string(&mut data_out).unwrap();
    assert_eq!(data_in, data_out);
}

#[test]
fn test_post_url() {
    setup();

    let data_in = "789";
    let my_key = "http://0.0.0.0:8081/v1/test_post_url";

    let client = reqwest::blocking::Client::new();
    let _res = client.post(format!("{my_key}/set/{data_in}"))
        .body("")
        .send().unwrap();

    let data_out = get_as_string(format!("{my_key}/get"));
    assert_eq!(data_in, data_out);
}

#[test]
fn test_post_body() {
    setup();

    let data_in = "987";
    let my_key = "http://0.0.0.0:8081/v1/test_post_body";

    let client = reqwest::blocking::Client::new();
    let _res = client.post(format!("{my_key}/set"))
        .body(data_in)
        .send().unwrap();

    let data_out = get_as_string(format!("{my_key}/get"));
    assert_eq!(data_in, data_out);
}

