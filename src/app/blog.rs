use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::*, hooks::use_params_map};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use gray_matter::{engine::YAML, Matter};
#[cfg(feature = "ssr")]
use pulldown_cmark::{Options, Parser};

#[cfg(feature = "ssr")]
use crate::highlight::highlight;

#[derive(Embed)]
#[folder = "blog"]
pub struct Assets;

#[component]
pub fn BlogWrapper() -> impl IntoView {
    view! {
        <h1 class="font-bold text-2xl text-center mb-8">
            <a href="/blog">"Hans Baker's Blog"</a>
        </h1>
        <div id="blog_content" class="w-[56rem] max-w-full text-left">
            <Outlet />
        </div>
    }
}

#[cfg(feature = "ssr")]
#[derive(Deserialize, Debug, Default)]
struct FrontMatter {
    title: String,
    author: String,
    date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMeta {
    name: String,
    title: String,
    author: String,
    date: DateTime<Utc>,
}

#[server]
pub async fn get_meta() -> Result<Vec<PostMeta>, ServerFnError> {
    let matter = Matter::<YAML>::new();
    let posts = Assets::iter()
        .map(|s| {
            let content = Assets::get(&s).expect("Should be able to get blog post");
            let content =
                &String::from_utf8(content.data.into()).expect("Couldn't parse blog post");
            let fm = matter
                .parse_with_struct::<FrontMatter>(&content)
                .ok_or(ServerFnError::new("Couldn't parse blog posts"))?;
            Ok(PostMeta {
                name: s[..s.len() - 3].to_string(),
                title: fm.data.title,
                author: fm.data.author,
                date: fm.data.date,
            })
        })
        .collect::<Result<Vec<PostMeta>, ServerFnError>>();
    posts.map(|pv| {
        let mut pv = pv;
        pv.sort_by(|a, b| b.date.cmp(&a.date));
        pv
    })
}

#[component]
pub fn BlogHome() -> impl IntoView {
    let posts = Resource::new(
        || (),
        move |_| async {
            let meta = get_meta().await.unwrap_or(Vec::new());
            meta
        },
    );
    view! {
        <Title text="Blog Home" />
        <div>
            <div>"$ ls -ot blog"</div>
            <Suspense>
                {move || Suspend::new(async move {
                    let posts = posts.await;
                    posts
                        .into_iter()
                        .map(|post| {
                            view! {
                                <div>
                                    <A attr:class="text-lg" href=post.name>
                                        "drw-r--r-- hans "
                                        <span>{format!("{}", post.date.format("%b %e %Y"))}</span>
                                        " "
                                        <span class="text-blue">{post.title}</span>
                                    </A>
                                </div>
                            }
                        })
                        .collect_view()
                })}
            </Suspense>
        </div>
    }
}

#[server]
pub async fn get_post(name: String) -> Result<String, ServerFnError> {
    let content = Assets::get(&name).ok_or(ServerFnError::new("Blog post not found"))?;
    let content = &String::from_utf8(content.data.into()).expect("Couldn't parse blog post");
    let parser = Parser::new_ext(&content, Options::all());
    let parser = highlight(parser);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    Ok(html_output)
}

#[component]
pub fn BlogPage() -> impl IntoView {
    let params = use_params_map();
    let post_name = move || params.get().get("post").unwrap_or_default();
    let post = Resource::new(
        move || post_name(),
        move |name| async {
            // take ownership of name
            let name = name;
            let name = format!("{}.md", name);
            let post = get_post(name).await;
            match post {
                Ok(s) => s,
                Err(e) => e.to_string(),
            }
        },
    );
    view! {
        <Title text="Blog Page" />
        <div>"$ cat "{post_name}".md"</div>
        <Suspense>
            {move || Suspend::new(async move {
                let post = post.await;
                view! {
                    <div inner_html=post></div>
                }
            })}
        </Suspense>
    }
}
