use actix_web::web;

pub mod cart;
pub mod products;
pub mod categories;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(cart::config)
            .configure(products::config)
            .configure(categories::config)
    );
}