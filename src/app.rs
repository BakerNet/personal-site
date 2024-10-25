mod commands;

use commands::CommandRes;
use leptos::{html, prelude::*};
use leptos_meta::*;
use leptos_router::{components::*, path};

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
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Header() -> impl IntoView {
    let input_ref = NodeRef::<html::Input>::new();
    let (text, set_text) = signal(None::<String>);
    let (is_err, set_is_err) = signal(false);

    let handle_res = move |res: CommandRes| match res {
        CommandRes::EmptyErr => {
            set_is_err(true);
            set_text(None);
        }
        CommandRes::Err(s) => {
            set_is_err(true);
            set_text(Some(s));
        }
        CommandRes::Redirect(_) => todo!(),
        CommandRes::Output(s) => {
            set_is_err(false);
            set_text(Some(s));
        }
        CommandRes::Nothing => {
            set_is_err(false);
            set_text(None);
        }
    };

    view! {
        <header class="bg-gray-800 shadow">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4">
                <div class="flex items-center justify-between">
                    <h1 class="text-2xl font-bold">

                        <span class=move || {
                            if is_err() { "text-red-500" } else { "text-green-500" }
                        }>"➜"</span>
                        " "
                        <span class="text-teal-400">"hansbaker.com"</span>
                        " "
                        <span class="text-blue-400">
                            <span>"git:("</span>
                            <span class="text-red-500">"main"</span>
                            <span>")"</span>
                        </span>
                        ""
                        <span class="text-yellow-400">"✗"</span>
                    </h1>
                    <form
                        class="flex-1 mx-4"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            let el = if let Some(el) = input_ref.get_untracked() {
                                el
                            } else {
                                set_is_err(true);
                                return;
                            };
                            let res = CommandRes::from(el.value().as_ref());
                            handle_res(res);
                            el.set_value("");
                        }
                    >
                        <div class="relative">
                            <input
                                node_ref=input_ref
                                type="text"
                                placeholder="Type a command (try 'help')"
                                class="w-full px-4 py-2 rounded-md border border-gray-700 bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                    </form>
                    <nav></nav>
                </div>
                {move || {
                    text.get()
                        .map(|s| {
                            view! {
                                <div class="mt-2 mr-4 p-2 bg-gray-700 rounded-md">
                                    <pre class="whitespace-pre-wrap">{s}</pre>
                                </div>
                            }
                        })
                }}
            </div>
        </header>
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
