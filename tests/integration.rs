use std::sync::Once;
use keyval_store::lib_main;
use actix_web::rt::System;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        let sys = System::new();
        sys.block_on(lib_main());
        //lib_main();
    });
}


#[test]
fn test_add() {
    setup();
    assert_eq!(1,1);
}
