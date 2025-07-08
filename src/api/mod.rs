// src/api/mod.rs
mod products;
mod categories;
mod cart;

pub use products::*;
pub use categories::*;
pub use cart::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(products::config)
            .configure(categories::config)
            .configure(cart::config)
    );
}