mod ascii;
mod avatar;
pub mod blog;
mod header;
mod homepage;
mod terminal;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use blog::{BlogHome, BlogPage, BlogWrapper};
use header::Header;
use homepage::HomePage;

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
                <link rel="stylesheet" href="/css/devicon.min.css" />
                <link rel="stylesheet" href="/css/blog.css" />
                <MetaTags />
            </head>
            <body class="flex flex-col font-mono min-h-screen bg-background text-foreground">
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
            <main class="flex flex-col flex-1 mx-auto max-w-full p-8">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=HomePage />
                    <ParentRoute path=path!("/blog") view=BlogWrapper>
                        <Route path=path!("/") view=BlogHome />
                        <Route path=path!("/:post") view=BlogPage />
                    </ParentRoute>
                    <Route path=path!("/cv") view=CVPage />
                </Routes>
            </main>
        </Router>
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
