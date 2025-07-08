use actix_web::web;

pub mod cart;
pub mod products;
pub mod categories;

use self::{cart::cart_config, products::products_config, categories::categories_config};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(products_config)
            .configure(categories_config)
            .configure(cart_config)
    );
}