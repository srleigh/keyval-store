// depends on crate reqwest = { version = "0.11.14", features = ["blocking"]}
use std::io::Read;
fn main()  {
    let data_in = "456";
    let my_key = "http://keyval.store/v1/my_key";
    let _res = reqwest::blocking::get(format!("{my_key}/set/{data_in}")).unwrap();
    let mut res = reqwest::blocking::get(format!("{my_key}/get")).unwrap();
    let mut data_out = String::new();
    res.read_to_string(&mut data_out).unwrap();
    assert_eq!(data_in, data_out);
}
