#[actix_web::main]
async fn main() -> std::io::Result<()> {
    keyval_store::lib_main().await
}
