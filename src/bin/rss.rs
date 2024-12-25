use personal_site::blog::get_meta;
use rss::{
    extension::atom::{AtomExtensionBuilder, Link},
    ChannelBuilder, GuidBuilder, ItemBuilder,
};
use std::fs::File;

#[tokio::main]
async fn main() {
    let posts = get_meta("".to_string())
        .await
        .expect("Should be able to get blog posts");
    let items = posts
        .into_iter()
        .map(|p| {
            let link = format!("https://hansbaker.com/blog/{}", p.name);
            let guid = GuidBuilder::default().value(&link).permalink(true).build();
            ItemBuilder::default()
                .title(p.title)
                .description(p.description)
                .author(p.author)
                .pub_date(p.date.to_rfc2822())
                .link(link)
                .guid(guid)
                .build()
        })
        .collect::<Vec<_>>();

    let mut atom_link = Link::default();
    atom_link.set_rel("self");
    atom_link.set_href("https://hansbaker.com/rss.xml");
    atom_link.set_mime_type("application/rss+xml".to_string());

    let channel = ChannelBuilder::default()
        .title("Hans Baker's Blog")
        .description("Insights and ramblings of a Software Engineering professional who has worn every hat, but mainly wants to code.")
        .link("https://hansbaker.com/blog")
        .language("en-us".to_string())
        .ttl("60".to_string())
        .atom_ext(AtomExtensionBuilder::default().links(vec![atom_link]).build())
        .items(items)
        .build();

    let file = File::create("public/rss.xml").expect("Should be able to create RSS feed file");
    channel
        .pretty_write_to(file, b' ', 2)
        .expect("Should be able to write RSS feed");
}
