use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use tera::{Context, Tera};

pub async fn login_page(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    // You can add any context variables here if needed

    match tera.render("login.html", &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
