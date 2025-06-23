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
        <Title text="Blog" />
        <div class="text-center mb-8">
            <h1 class="font-bold text-3xl lg:text-4xl mb-4 section-content">
                <a
                    href="/blog"
                    on:click=move |_| clicked.notify()
                    class="hover:text-purple transition-colors duration-200"
                >
                    "Hans Baker's Blog"
                </a>
                <a
                    href="https://hansbaker.com/rss.xml"
                    target="_blank"
                    class="relative top-1 ml-4 text-brightYellow hover:text-yellow transition-colors duration-200"
                    aria-label="RSS Feed"
                >
                    <i class="extra-rss" />
                </a>
            </h1>
            <div class="max-w-2xl mx-auto text-lg font-medium text-muted section-content">
                "Insights and ramblings of a Software Engineering professional who has worn many hats, but mainly wants to code."
            </div>
        </div>
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
        <div class="mb-6">
            <form
                class="flex flex-col sm:flex-row gap-3 items-start sm:items-center"
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
                <label for="blog_grep" class="font-medium text-cyan whitespace-nowrap">
                    "🔍 Search (regex):"
                </label>
                <div class="flex-grow w-full sm:max-w-md">
                    <input
                        id="blog_grep"
                        class="w-full px-4 py-2 rounded-md border border-muted focus:outline-none focus:ring-2 focus:ring-cyan focus:border-cyan bg-background text-foreground placeholder-muted transition-all duration-200"
                        node_ref=input_ref
                        placeholder="Enter search pattern..."
                    />
                </div>
                <button
                    type="submit"
                    class="px-4 py-2 bg-cyan/20 hover:bg-cyan/30 text-cyan rounded-md border border-cyan/30 transition-all duration-200 whitespace-nowrap"
                >
                    "Search"
                </button>
            </form>
        </div>
        <div>
            <div class="bg-black/40 border border-muted/30 p-3 rounded-md font-mono text-sm backdrop-blur-sm">
                <span class="text-green">$</span>
                <span class="text-foreground ml-2">
                    {move || {
                        let s = search.get();
                        if s.is_empty() {
                            "ls -lt blog".to_string()
                        } else {
                            format!("grep -Eil '{s}' blog/* | xargs ls -lt")
                        }
                    }}
                </span>
            </div>
            <div class="my-4"></div>
            <Transition fallback=move || {
                view! {
                    <div class="space-y-4">
                        <div class="loading-skeleton h-8 rounded"></div>
                        <div class="loading-skeleton h-6 rounded w-3/4"></div>
                        <div class="loading-skeleton h-8 rounded"></div>
                        <div class="loading-skeleton h-6 rounded w-2/3"></div>
                        <div class="loading-skeleton h-8 rounded"></div>
                        <div class="loading-skeleton h-6 rounded w-4/5"></div>
                    </div>
                }
            }>
                {move || Suspend::new(async move {
                    let posts = posts.await;
                    view! {
                        <div class="space-y-4 section-content">
                            {posts
                                .into_iter()
                                .map(|post| {
                                    view! {
                                        <div class="mb-4 hover:bg-brightBlack/20 p-2 rounded-md transition-colors duration-200">
                                            <A attr:class="text-lg leading-tight block" href=post.name>
                                                <div>
                                                    "drw-r--r-- hans "
                                                    <span>{format!("{}", post.date.format("%b %e %Y"))}</span>
                                                    " " <span class="text-blue font-medium">{post.title}</span>
                                                </div>
                                                <div class="mt-1">
                                                    {post
                                                        .tags
                                                        .iter()
                                                        .map(|s| {
                                                            view! {
                                                                <span class="rounded-md px-2 py-1 bg-brightBlack mr-2 text-sm">
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
                                .collect_view()}
                        </div>
                    }
                })}
            </Transition>
        </div>
    }
}

#[server(input = GetUrl)]
pub async fn get_post_server(name: String) -> Result<Post, ServerFnError> {
    let name = format!("{name}.md");
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
            <div class="bg-black/40 border border-muted/30 p-3 rounded-md font-mono text-sm backdrop-blur-sm mb-6">
                <span class="text-green">$</span>
                <span class="text-foreground ml-2">"cat "{post_name}".md"</span>
            </div>
            <Suspense>
                {move || Suspend::new(async move {
                    let post = post.await;
                    post.map(|p| {
                        view! {
                            <div class="mb-6 p-4 bg-brightBlack/20 rounded-md border border-muted/30">
                                <div class="flex flex-wrap items-center gap-4 text-sm">
                                    <span class="text-cyan font-medium">
                                        "👤 " {p.meta.author}
                                    </span>
                                    <span class="text-yellow font-medium">
                                        "📅 " {p.meta.date.format("%b %e, %Y").to_string()}
                                    </span>
                                    <div class="flex flex-wrap gap-1">
                                        <span class="text-green font-medium">"🏷️ "</span>
                                        {p
                                            .meta
                                            .tags
                                            .into_iter()
                                            .map(|tag| {
                                                view! {
                                                    <span class="bg-green/20 text-green px-2 py-1 rounded text-xs">
                                                        {tag}
                                                    </span>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                </div>
                            </div>
                            <article class="prose prose-invert max-w-none">
                                <div inner_html=p.content></div>
                            </article>
                        }
                    })
                })}
            </Suspense>
        </div>
    }
}
