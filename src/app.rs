mod header;
mod terminal;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use header::Header;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <meta name="color-scheme" content="dark" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <link rel="stylesheet" id="leptos" href="/pkg/personal-site.css" />
                <MetaTags />
            </head>
            <body class="font-mono">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // sets the document title
        <Title formatter=|title| format!("Hans Baker - {title}") />

        // content for this welcome page
        <Router>
            <Header />
            <main class="flex flex-col flex-grow justify-center items-center mx-auto w-full max-w-7xl">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=HomePage />
                    <ParentRoute path=path!("/blog") view=BlogWrapper>
                        <Route path=path!("/") view=BlogHome />
                    </ParentRoute>
                    <Route path=path!("/cv") view=CVPage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <Title text="About Me" />
        <h1 class="font-bold text-2xl text-center">"Hans Baker"</h1>
        <div>TODO</div>
    }
}

#[component]
fn BlogWrapper() -> impl IntoView {
    view! {
        <h1 class="font-bold text-2xl text-center">"Blog"</h1>
        <Outlet />
    }
}

#[component]
fn BlogHome() -> impl IntoView {
    view! {
        <Title text="Blog Home" />
        <div>TODO</div>
    }
}

#[component]
fn CVPage() -> impl IntoView {
    view! {
        <Title text="CV / Resume" />
        <h1 class="font-bold text-2xl text-center">"CV / Resume"</h1>
        <div>TODO</div>
    }
}