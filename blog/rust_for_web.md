---
title: Writing Rust for Web is a Pleasure
description: Leptos makes writing fullstack web applications a pleasure - learn about how Leptos provides a developer experience on par with the TypeScript ecosystem when writing Rust for the web.
author: Hans Baker
date: 2024-12-24T12:00:00Z
tags:
  - rust
  - programming
---
# Writing Rust for Web is a Pleasure

**Leptos and Rust make writing fullstack web applications a pleasure**

When people hear about using Rust for frontend development, their first reaction is often, "Why would you?".  JavaScript/TypeScript has a wonderfully active community, is the native language of the browser, and has massive economic backing to support improving it's developer experience.

There really isn't a hole in the TypeScript ecosystem these days, so what reason would one have for looking elsewhere?

For me the answer is simple:  I like the act of writing Rust more than the act of writing TypeScript, and I think it's pretty neat that it's even possible to use Rust for Web frontend.

I was surprised to find out, the developer experience of writing fullstack applications in Rust is actually fantastic thanks to the [Leptos project](https://github.com/leptos-rs/leptos).

## What is Leptos

Leptos...
- is a web framework for building CSR (Client-side rendered), SSR (Server-side rendered), or SSG (Static site generation) websites and web apps
- is written in Rust
- compiles to WASM ([WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly)) for the frontend
- uses fine-grained reactivity, with an approach most similar to [SolidJS](https://www.solidjs.com/) in the JS/TS world
- includes a `view!` macro which allows for writing JSX-like syntax
- has integrations for the most popular backend Rust frameworks
- includes macros for declaring server functions, like you get in popular fullstack frameworks such as [Next.js](https://nextjs.org/docs/app/building-your-application/data-fetching/server-actions-and-mutations)
- has a small but active community building libraries (shoutout to [leptos-use](https://github.com/synphonyte/leptos-use) in particular)
- is quite performant ([source](https://krausest.github.io/js-framework-benchmark/))
- ... and much more

Leptos isn't the only library in the Rust frontend or fullstack space.  Some alternatives include:
- [Yew](https://github.com/yewstack/yew) - which is more similar to React in architecture
- [Dioxus](https://github.com/DioxusLabs/dioxus) - which targets building for both web and native
- [Sycamore](https://github.com/sycamore-rs/sycamore) - which is pretty similar to Leptos in architecture but doesn't use JSX-like view syntax

## Leptos Has DX

Let's first take a look at the root of the code used to generate _this very site_:

```rust
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
        <Title formatter=|title| format!("Hans Baker - {title}") />

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
```

Now tell me that isn't beautifully clear code!  Anyone who has worked in a JSX-based framework could easily read this and understand what is going on.

This site uses SSR architecture, so the `Routes` you see in the code above work on both the server-side for initial requests and client-side for navigation.

### Cargo-leptos

You may notice `AutoReload` in the HTML head of the `shell` code above - this allows you to use **hot reloading** during development, just as you would expect when working with `webpack` or `vite`.

Leptos comes with a really wonderful CLI tool [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) which handles development and production builds - this is what powers the hot reloads.  It even comes equipped with [SCSS](https://sass-lang.com/) and [Tailwind](https://tailwindcss.com/) compilers built in!

By just running `cargo leptos watch` you get up and running with auto-reloading localhost for the iteration loop you are used to when working with modern frameworks.

### Sharing Code is Great

Writing both the frontend and the backend in the same language is really wonderful due to the ability to share data structures and functions directly - without the need for code duplication.

The client can run the same validation logic as the server - the same function can be used to prevent user submission on the frontend, and validation before saving on the backend.

You also have access to most crates on both frontend and backend.  One example of a crate which provides code sharing benefits is `serde` to serialize and deserialize data.

See the use of `Serialize` and `Deserialize` in the `derive` macro below:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMeta {
    name: String,
    title: String,
    author: String,
    date: DateTime<Utc>,
    tags: Vec<String>,
}
```

Simply deriving these traits allows for any data structure to easily be sent and received between client and server using whichever serialization codec you prefer.

### Server-only and Client-only code

Rust has a concept of [conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html) which allows you to have different code included when compiling for different contexts.  The `#[cfg(...)]` macro provides a flexible framework when separating client-side code from server-side code.  One can use compiler flags at any level of granularity from modules to individual lines of logic.

Let's look at some examples!

In the following code snippet, I use compiler flags to only include the `highlight` module (which generates the syntax highlighting in this blog post) in the server-side code, and to only include the hydration script in the client-side code:
```rust
pub mod app;
#[cfg(feature = "ssr")]
mod highlight;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

In this next code snippet, I use compiler flags to only try to access the `Window` on the client side:
```rust
#[cfg(not(feature = "ssr"))]
let origin = { window().location().origin().unwrap_or_default() };
#[cfg(feature = "ssr")]
let origin = String::new();
let url = format!("{}/game/{}", origin, game_id);
```

Leptos also comes with an excellent `#[server]` macro built in.  Similar to how React now has a ["use server" directive](https://react.dev/reference/rsc/use-server), which is used for server actions in Next.js and other framework, Leptos has `#[server]` macro.

The following code snippet demonstrates declaring a server action in Leptos:
```rust
#[server]
pub async fn get_active_games() -> Result<Vec<SimpleGameInfo>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let active_games = game_manager.get_active_games().await;

    Ok(active_games.into_iter().map(SimpleGameInfo::from).collect())
}
```

This server action can then be fetched within a component inside a `Resource` or invoked by a form using `ActionForm`.

Under the hood, the `#[server]` macro expands to a server-side function & web handler and a client-side function (which makes an HTTP request).  Under the hood, this is utilizing the conditional compilation as discussed above.  To me, this system is much more intuitive than `"use server"` and `"use client"` in the JavaScript/TypeScript world.

## The Elephant in the Room

While I would love to say developing fullstack web apps with Leptos is all roses, I'd be leaving out the one **glaring** downside:

Compilation times can get _quite long_.

When first starting out with a Leptos project, compilation will only take a few seconds if dependencies are cached.  But one of my projects ([minesweeper-io](https://github.com/BakerNet/minesweeper-io)) takes a full minute to compile the frontend in dev mode.  This means any change, including small things like updating a TailwindCSS class will require a minute long feedback cycle.

Waiting a minute for a TailwindCSS class change to reflect in the browser can disrupt your flow and make quick iterations frustrating.  Of course one could just not use TailwindCSS and the CSS situation would be better (CSS _can_ be hot-reloaded), but any logic change will still have slower iteration loops.

I mentioned earlier that `cargo-leptos` supports hot reloading, but the WASM binary itself can't be code-split ([yet at least](https://github.com/rustwasm/wasm-bindgen/issues/3939)) - so hot reloading does not work at a module level like it can with JavaScript.

I wouldn't blame anyone for deciding Rust was not worth it due to compilation times.

### Improving Compile Times

Compile times _can_ be improved some by using cargo [workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) to split up code.  Because cargo caches dependencies, you will only have to recompile the workspace members that you directly changed during any iteration loop.

But there's really only so much you can improve with this approach.

There are also some compiler-level changes one can make to improve compile times a lot - see Leptos maintainer [benwis](https://github.com/benwis)' [blog post](https://benw.is/posts/how-i-improved-my-rust-compile-times-by-seventy-five-percent) on the matter.

## Conclusion

Leptos really reignited my passion for web development by allowing me to use my favorite programming language across the stack.  Greg Johnston ([gbj](https://github.com/gbj)) has made a wonderful framework and his dedication to the project and the community that has built around it is quite commendable.

I have really only focused on the developer experience here, but there are some other downsides to using WASM for web development.  To name a few:  code splitting isn't feasible yet, accessing browser APIs is a bit more cumbersome, use of any JS dependencies will require some setup, etc.

While Rust for the web isn't perfect, the developer experience is much better than I ever would have expected.  If you were on the fence about trying Rust for web, definitely give it a try.  I think you'll be surprised how rich the experience is.

Head over to the [Leptos Book](https://book.leptos.dev/) to get started.
