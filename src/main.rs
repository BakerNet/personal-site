#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use personal_site::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    // logging::log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(all(feature = "rss", not(feature = "ssr")))]
#[tokio::main]
async fn main() {
    use personal_site::blog::get_meta;
    use personal_site::rss::build_channel;
    use std::fs::File;

    let posts = get_meta("".to_string())
        .await
        .expect("Should be able to get blog posts");

    let channel = build_channel(posts);

    let file = File::create("public/rss.xml").expect("Should be able to create RSS feed file");
    channel
        .pretty_write_to(file, b' ', 2)
        .expect("Should be able to write RSS feed");
}

#[cfg(not(any(feature = "ssr", feature = "rss")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
