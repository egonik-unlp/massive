pub mod api;
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        extract::FromRef,
        routing::{get, post},
        Router,
    };
    use leptos::prelude::*;
    use leptos::{config::LeptosOptions, logging::log};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use massive::app::*;
    use massive::otro_upload::AppState;
    use tower_http::services::ServeDir;
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("Routes {:?}", routes);
    // let state = todo!();
    log!("Routes {:?}", routes);
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        ingredient_vec: None,
    };
    let leptos_for_router_ref = leptos_options.clone(); // para &...
    let app_state_for_ctx = app_state.clone(); // para provide_context
    let leptos_for_shell = leptos_options.clone(); // para el closure shell
    let leptos_for_state = leptos_options.clone(); // para el closure shell
    let app = Router::new()
        .nest_service("/uploads", ServeDir::new("./uploads"))
        .leptos_routes_with_context(
            &leptos_for_router_ref,
            routes,
            move || provide_context(app_state_for_ctx.clone()),
            move || shell(leptos_for_shell.clone()),
        )
        // Expose Leptos server function endpoints under the `/api` prefix
        // // .route("/images/", ServeDir::new("uploads"))
        // .leptos_routes(&leptos_options, routes, {
        //     let leptos_options = leptos_options.clone();
        //     move || shell(leptos_options.clone())
        // })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_for_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
