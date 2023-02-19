#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port:u16 = {
        if args.len() > 1{
            args[1].parse().unwrap()
        }
        else{
            8080
        }
    };
    keyval_store::lib_main(port).await
}
