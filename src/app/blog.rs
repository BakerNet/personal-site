use chrono::{DateTime, Utc};
use dashmap::DashMap;
use leptos::{html::Input, prelude::*};
use leptos_meta::Title;
use leptos_router::{components::*, hooks::*};
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
use regex::RegexBuilder;

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
    let clicked = ArcTrigger::new();
    provide_context(clicked.clone());
    view! {
        <h1 class="font-bold text-2xl text-center mb-8">
            <a href="/blog" on:click=move |_| clicked.notify()>
                "Hans Baker's Blog"
            </a>
        </h1>
        <div class="w-full max-w-4xl mx-auto text-left">
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
pub async fn get_meta(pattern: String) -> Result<Vec<PostMeta>, ServerFnError> {
    let cache = &*GLOBAL_META_CACHE;
    let is_base = pattern == "";
    if is_base {
        if let Some(r) = cache.get(&pattern) {
            return Ok(r.clone());
        }
    }
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .map_err(|e| ServerFnError::new(format!("Couldn't parse regex: {}", e)))?;
    let matter = Matter::<YAML>::new();
    let posts = Assets::iter()
        .map(|s| {
            let content = Assets::get(&s).expect("Should be able to get blog post");
            (
                s,
                String::from_utf8(content.data.into()).expect("Couldn't parse blog post"),
            )
        })
        .filter(
            |(_, content)| {
                if is_base {
                    true
                } else {
                    re.is_match(content)
                }
            },
        )
        .map(|(s, content)| {
            let fm = matter
                .parse_with_struct::<FrontMatter>(&content)
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
    let posts = posts.map(|pv| {
        let mut pv = pv;
        pv.sort_by(|a, b| b.date.cmp(&a.date));
        pv
    });
    if is_base {
        cache.insert(pattern, posts.clone().unwrap_or_default());
    }

    posts
}

#[component]
pub fn BlogHome() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let input_ref = NodeRef::<Input>::new();
    let posts = Resource::new(search, move |search| async move {
        let cache = &*GLOBAL_META_CACHE;
        if let Some(s) = cache.get(&search) {
            return (*s).clone();
        }
        let meta = get_meta(search.clone()).await.unwrap_or(Vec::new());
        // only cache all searches on the browser
        #[cfg(feature = "hydrate")]
        cache.insert(search, meta.clone());
        meta
    });

    let header_clicked = expect_context::<ArcTrigger>();
    Effect::watch(
        move || header_clicked.track(),
        move |_, _, _| {
            let el = if let Some(el) = input_ref.get_untracked() {
                el
            } else {
                return;
            };
            set_search(String::new());
            el.set_value("");
        },
        false,
    );

    view! {
        <Title text="Blog Home" />
        <div>
            <form
                class="mb-4 flex flex-row space-x-2"
                on:submit=move |ev| {
                    ev.prevent_default();
                    let el = if let Some(el) = input_ref.get_untracked() {
                        el
                    } else {
                        return;
                    };
                    set_search(el.value());
                }
            >
                <label for="blog_grep" class="font-md">
                    "Search (regex): "
                </label>
                <input
                    id="blog_grep"
                    class="flex-grow max-w-72 min-w-12 px-2 rounded-md border focus:outline-none focus:ring-2 focus:ring-brightBlack bg-background text-foreground"
                    node_ref=input_ref
                    placeholder="match pattern"
                />
            </form>
        </div>
        <div>
            <div class="bg-black p-2 rounded-md">
                {move || {
                    let s = search.get();
                    if s == "" {
                        "$ ls -lt blog".to_string()
                    } else {
                        format!("$ grep -Eil '{}' blog/* | xargs ls -lt", s)
                    }
                }}
            </div>
            <br />
            <Transition>
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
                                                            {s.to_string()}
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
            </Transition>
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
        let post_data = get_post(name.clone()).await;
        cache.insert(name, post_data.clone());
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
                                        .into_iter()
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
