use crate::handlers::{auth, user};
use crate::middlewares::api_auth;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth").service(
                    web::resource("/token")
                        .route(web::post().to(auth::login))
                        .route(web::delete().to(auth::logout).wrap(api_auth::CheckLogin)),
                ),
            )
            .service(
                web::scope("/user")
                    .wrap(api_auth::CheckLogin)
                    .service(web::resource("/profile").route(web::get().to(user::get_profile)))
                    .service(web::resource("/password").route(web::put().to(user::update_password))),
            )
            .service(
                web::scope("/admin")
                    .wrap(api_auth::CheckAdminLogin)
                    .service(
                        web::scope("/users")
                            .service(
                                web::resource("")
                                    .route(web::get().to(user::list_users))
                                    .route(web::post().to(user::add_user)),
                            )
                            .service(web::scope("/{user_id}").service(
                                web::resource("").route(web::put().to(user::update_user)),
                            )),
                    ),
            ),
    );
}
