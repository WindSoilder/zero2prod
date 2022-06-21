use zero2prod::get_server;

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let server = get_server();
    server.listen("127.0.0.1:8000").await?;
    Ok(())
}
