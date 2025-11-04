use actix_web::App;
use actix_web::HttpServer;
use actix_web::middleware;
use actix_web::web::ThinData;
use dotenvy::dotenv;
use doup::database::postgresql::PgPool;
use doup::database::postgresql::{get_pool, run_migration};
use doup::utils::logger::init as init_logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    dotenv().ok();

    run_migration().await;
    let pool: PgPool = get_pool().await;

    log::info!("Start actix-web server on 127.0.0.1:8080...");

    HttpServer::new(move || {
        App::new()
            .app_data(ThinData(pool.clone()))
            .configure(doup::routes::register_all)
            .wrap(middleware::NormalizePath::trim())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
