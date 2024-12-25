use leptos::{html::Input, prelude::*};
use leptos_meta::Title;
use leptos_router::{components::*, hooks::*};
use server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use crate::blog::{get_meta, get_post};
use crate::blog::{Post, PostMeta, GLOBAL_META_CACHE, GLOBAL_POST_CACHE};

#[component]
pub fn BlogWrapper() -> impl IntoView {
    let clicked = ArcTrigger::new();
    provide_context(clicked.clone());
    view! {
        <h1 class="font-bold text-2xl text-center mb-8">
            <a href="/blog" on:click=move |_| clicked.notify()>
                "Hans Baker's Blog"
            </a>
            <a
                href="https://hansbaker.com/rss.xml"
                target="_blank"
                class="relative top-1 ml-4 text-white"
            >
                <i class="extra-rss" />
            </a>
        </h1>
        <div class="w-full max-w-4xl mx-auto text-left">
            <Outlet />
        </div>
    }
}

#[server(input = GetUrl)]
pub async fn get_meta_server(pattern: String) -> Result<Vec<PostMeta>, ServerFnError> {
    get_meta(pattern)
        .await
        .ok_or(ServerFnError::new("Couldn't parse blog posts"))
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
        let meta = get_meta_server(search.clone()).await.unwrap_or(Vec::new());
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
                    if s.is_empty() {
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

#[server(input = GetUrl)]
pub async fn get_post_server(name: String) -> Result<Post, ServerFnError> {
    let name = format!("{}.md", name);
    get_post(name)
        .await
        .ok_or(ServerFnError::new("Couldn't get blog post"))
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
            return (*s)
                .clone()
                .ok_or(ServerFnError::new("Couldn't get blog post"));
        }
        let post_data = get_post_server(name.clone()).await;
        cache.insert(name, post_data.clone().ok());
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
