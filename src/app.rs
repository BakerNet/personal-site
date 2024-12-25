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
                <link rel="stylesheet" href="/css/extra-icons.css" />
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
            <main class="flex flex-col flex-1 w-full p-8 lg:p-12">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=HomePage />
                    <ParentRoute path=path!("/blog") view=BlogWrapper>
                        <Route path=path!("/") view=BlogHome />
                        <Route path=path!("/:post") view=BlogPage />
                    </ParentRoute>
                    <Route path=path!("/cv") view=CVPage />
                </Routes>
            </main>
            <Footer />
        </Router>
    }
}

#[component]
fn CVPage() -> impl IntoView {
    view! {
        <Title text="CV / Resume" />
        <div class="grid mx-auto">
            <h1 class="font-bold text-2xl text-center mb-8">"CV / Resume"</h1>
            <div class="text-center">
                <div class="bg-brightBlack text-yellow font-bold px-4 py-2 mb-4 inline-block rounded-lg border-4 border-yellow">
                    <span class="animate-blink">"🚧"</span>
                    " Under Construction "
                    <span class="animate-blink">"🚧"</span>
                </div>
            </div>
            <div class="text-center">
                "In the meantime, check out my "
                <a class="text-blue" href="https://linkedin.com/in/hansbaker">
                    "Linked"
                    <i class="devicon-linkedin-plain" />
                </a>
            </div>
        </div>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-black">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4 leading-tight">
                <div class="grid grid-cols-1 lg:grid-cols-3">
                    <div class="order-first text-center lg:order-last lg:text-right">
                        <a
                            class="text-2xl text-brightBlue mx-1"
                            href="https://bsky.app/profile/hansbaker.com"
                        >
                            <i class="extra-bluesky" />
                        </a>
                        <a class="text-2xl text-white mx-1" href="https://github.com/BakerNet">
                            <i class="devicon-github-plain" />
                        </a>
                        <a class="text-2xl text-blue mx-1" href="https://linkedin.com/in/hansbaker">
                            <i class="devicon-linkedin-plain" />
                        </a>
                    </div>
                    <div class="text-center">"© Hans Baker, 2024. All rights reserved"</div>
                    <div class="order-last text-center lg:order-first lg:text-left">
                        "Built with Rust & Leptos"
                    </div>
                </div>
            </div>
        </footer>
    }
}
