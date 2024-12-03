use chrono::{DateTime, Utc};
use dashmap::DashMap;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::*, hooks::use_params_map};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use server_fn::codec::GetUrl;
use std::sync::LazyLock;

#[cfg(feature = "ssr")]
use gray_matter::{engine::YAML, Matter};
#[cfg(feature = "ssr")]
use leptos::logging;
#[cfg(feature = "ssr")]
use pulldown_cmark::{Options, Parser};

#[cfg(feature = "ssr")]
use crate::highlight::highlight;

static GLOBAL_POST_CACHE: LazyLock<DashMap<String, Result<Post, ServerFnError>>> =
    LazyLock::new(DashMap::new);
static GLOBAL_META_CACHE: LazyLock<DashMap<String, Vec<PostMeta>>> = LazyLock::new(DashMap::new);

#[derive(Embed)]
#[folder = "blog"]
#[cfg_attr(feature = "hydrate", metadata_only = true)]
pub struct Assets;

#[component]
pub fn BlogWrapper() -> impl IntoView {
    view! {
        <h1 class="font-bold text-2xl text-center mb-8">
            <a href="/blog">"Hans Baker's Blog"</a>
        </h1>
        <div class="w-[80rem] max-w-full mx-auto text-left">
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
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMeta {
    name: String,
    title: String,
    author: String,
    date: DateTime<Utc>,
    tags: Vec<String>,
}

#[server(input = GetUrl)]
pub async fn get_meta() -> Result<Vec<PostMeta>, ServerFnError> {
    let cache = &*GLOBAL_META_CACHE;
    Ok(cache
        .entry("".to_string())
        .or_insert_with(move || {
            let matter = Matter::<YAML>::new();
            let posts = Assets::iter()
                .map(|s| {
                    let content = Assets::get(&s).expect("Should be able to get blog post");
                    let content =
                        &String::from_utf8(content.data.into()).expect("Couldn't parse blog post");
                    let fm = matter
                        .parse_with_struct::<FrontMatter>(content)
                        .ok_or_else(|| {
                            logging::error!("Unable to parse meta for {}", s);
                            ServerFnError::new("Couldn't parse blog posts")
                        })?;
                    Ok(PostMeta {
                        name: s[..s.len() - 3].to_string(),
                        title: fm.data.title,
                        author: fm.data.author,
                        date: fm.data.date,
                        tags: fm.data.tags,
                    })
                })
                .collect::<Result<Vec<PostMeta>, ServerFnError>>();
            posts
                .map(|pv| {
                    let mut pv = pv;
                    pv.sort_by(|a, b| b.date.cmp(&a.date));
                    pv
                })
                .unwrap_or_default()
        })
        .to_vec())
}

#[component]
pub fn BlogHome() -> impl IntoView {
    let posts = Resource::new(
        || (),
        move |_| async {
            let cache = &*GLOBAL_META_CACHE;
            if let Some(s) = cache.get(&"".to_string()) {
                return (*s).clone();
            }
            let meta = get_meta().await.unwrap_or(Vec::new());
            cache.insert("".to_string(), meta.clone());
            meta
        },
    );
    view! {
        <Title text="Blog Home" />
        <div>
            <div class="bg-black p-2 rounded-md">"$ ls -lt blog"</div>
            <br />
            <Suspense>
                {move || Suspend::new(async move {
                    let posts = posts.await;
                    posts
                        .into_iter()
                        .map(|post| {
                            view! {
                                <div class="mb-4">
                                    <A attr:class="text-lg leading-tight" href=post.name>
                                        <div>
                                            "drw-r--r-- hans "
                                            <span>{format!("{}", post.date.format("%b %e %Y"))}</span>
                                            " " <span class="text-blue">{post.title}</span>
                                        </div>
                                        <div>
                                            {post
                                                .tags
                                                .iter()
                                                .map(|s| {
                                                    view! {
                                                        <span class="rounded-md px-1 bg-brightBlack mr-2">
                                                            "#"{s.to_string()}
                                                        </span>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    meta: PostMeta,
    content: String,
}

#[server(input = GetUrl)]
pub async fn get_post(name: String) -> Result<Post, ServerFnError> {
    let name = format!("{}.md", name);
    let content = Assets::get(&name).ok_or(ServerFnError::new("Blog post not found"))?;

    let cache = &*GLOBAL_POST_CACHE;
    cache
        .entry(name.clone())
        .or_insert_with(move || {
            let matter = Matter::<YAML>::new();
            let content =
                &String::from_utf8(content.data.into()).expect("Couldn't parse blog post");

            let fm = matter
                .parse_with_struct::<FrontMatter>(content)
                .ok_or_else(|| {
                    logging::error!("Unable to parse meta for {}", name);
                    ServerFnError::new("Couldn't parse blog posts")
                })?;
            let meta = PostMeta {
                name: name[..name.len() - 3].to_string(),
                title: fm.data.title,
                author: fm.data.author,
                date: fm.data.date,
                tags: fm.data.tags,
            };

            let parser = Parser::new_ext(content, Options::all());
            let parser = highlight(parser);

            // Write to a new String buffer.
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);

            Ok(Post {
                meta,
                content: html_output,
            })
        })
        .clone()
}

#[component]
pub fn BlogPage() -> impl IntoView {
    let params = use_params_map();
    let post_name = move || params.get().get("post").unwrap_or_default();
    let post = Resource::new(post_name, move |name| async {
        // take ownership of name
        let name = name;
        let cache = &*GLOBAL_POST_CACHE;
        if let Some(s) = cache.get(&name) {
            return (*s).clone();
        }
        let post_data = get_post(name).await;
        cache.insert("".to_string(), post_data.clone());
        post_data
    });
    view! {
        <Title text="Blog Page" />
        <div id="blog_content">
            <div class="bg-black p-2 rounded-md">"$ cat "{post_name}".md"</div>
            <br />
            <Suspense>
                {move || Suspend::new(async move {
                    let post = post.await;
                    post.map(|p| {
                        view! {
                            <div>
                                {format!(
                                    "{} | {} | tags: {}",
                                    p.meta.author,
                                    p.meta.date.format("%b %e, %Y"),
                                    p
                                        .meta
                                        .tags
                                        .iter()
                                        .map(|s| format!("#{}", s))
                                        .fold(
                                            String::new(),
                                            |acc, s| {
                                                if acc.is_empty() { s } else { format!("{}, {}", acc, s) }
                                            },
                                        ),
                                )}
                            </div>
                            <br />
                            <div inner_html=p.content></div>
                        }
                    })
                })}
            </Suspense>
        </div>
    }
}
