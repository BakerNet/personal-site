use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use thiserror::Error;

#[cfg(any(feature = "ssr", feature = "rss"))]
use gray_matter::{engine::YAML, Matter};
#[cfg(any(feature = "ssr", feature = "rss"))]
use pulldown_cmark::{Options, Parser};
#[cfg(any(feature = "ssr", feature = "rss"))]
use regex::RegexBuilder;

#[cfg(any(feature = "ssr", feature = "rss"))]
use crate::highlight::highlight;

pub static GLOBAL_POST_CACHE: LazyLock<DashMap<String, Option<Post>>> = LazyLock::new(DashMap::new);
pub static GLOBAL_META_CACHE: LazyLock<DashMap<String, Vec<PostMeta>>> =
    LazyLock::new(DashMap::new);

#[derive(Embed)]
#[folder = "blog"]
#[cfg_attr(feature = "hydrate", metadata_only = true)]
pub struct Assets;

#[cfg(any(feature = "ssr", feature = "rss"))]
#[derive(Deserialize, Debug, Default)]
struct FrontMatter {
    title: String,
    description: String,
    author: String,
    date: DateTime<Utc>,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMeta {
    pub name: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Error, Debug, Clone)]
pub enum BlogError {
    #[error("Blog post not found")]
    NotFound,
    #[error("Couldn't parse blog posts")]
    ParseError,
}

#[cfg(any(feature = "ssr", feature = "rss"))]
pub async fn get_meta(pattern: String) -> Option<Vec<PostMeta>> {
    let cache = &*GLOBAL_META_CACHE;
    let is_base = pattern.is_empty();
    if is_base {
        if let Some(r) = cache.get(&pattern) {
            return Some(r.clone());
        }
    }
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .ok()?;
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
            let fm = matter.parse_with_struct::<FrontMatter>(&content)?;
            Some(PostMeta {
                name: s[..s.len() - 3].to_string(),
                title: fm.data.title,
                description: fm.data.description,
                author: fm.data.author,
                date: fm.data.date,
                tags: fm.data.tags,
            })
        })
        .collect::<Option<Vec<PostMeta>>>();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub meta: PostMeta,
    pub content: String,
}

#[cfg(any(feature = "ssr", feature = "rss"))]
pub async fn get_post(name: String) -> Option<Post> {
    let content = Assets::get(&name)?;

    let cache = &*GLOBAL_POST_CACHE;
    cache
        .entry(name.clone())
        .or_insert_with(move || {
            let matter = Matter::<YAML>::new();
            let content =
                &String::from_utf8(content.data.into()).expect("Couldn't parse blog post");

            let fm = matter.parse_with_struct::<FrontMatter>(content)?;
            let meta = PostMeta {
                name: name[..name.len() - 3].to_string(),
                title: fm.data.title,
                description: fm.data.description,
                author: fm.data.author,
                date: fm.data.date,
                tags: fm.data.tags,
            };

            let parser = Parser::new_ext(content, Options::all());
            let parser = highlight(parser);

            // Write to a new String buffer.
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);

            Some(Post {
                meta,
                content: html_output,
            })
        })
        .clone()
}
