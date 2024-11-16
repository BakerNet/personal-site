use leptos::prelude::*;
use leptos_meta::Title;

use super::avatar::{Avatar, InfoBlock};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="About Me" />
        <section class="flex flex-col lg:flex-row justify-center items-center lg:mt-8 xl:mt-16">
            <div class="max-w-full mt-4 mr-4">
                <Avatar />
            </div>
            <div class="max-w-full overflow-visible overflow-x-auto mt-4">
                <InfoBlock />
            </div>
        </section>
        <section class="w-full max-w-4xl pt-12">
            <h2 class="text-xl my-8">"Hi, I'm Hans - welcome to my personal site."</h2>
            <p class="text-md mb-2">
                "I'm a professional working in software development since 2016.  I've been all the way up the career ladder from Junior Software Engineer to Senior Software Engineer to Engineering Manager to Director of Engineering and now Principal Architect.  I take a lot of pride in my work and always strive for excellence.  I respect my coworkers and my users."
            </p>
            <p class="text-md mb-2">
                "For the past 7 years, I've been working at Multi Media LLC.  We run one of the largest sites on the internet, with over 2 billion monthly sessions and petabtye scale data.  Our platform is a highly interactive live streaming web app.  I've worked all across the stack as an engineer - frontend, backend, microservice, and tooling.  I'm very proud of what I've accomplished so far at Multi Media LLC and can't wait to see what the future holds."
            </p>
            <p class="text-md mb-2">
                "Please feel free to reach out to me for inquiries, mentorship, or opportunities."
            </p>
        </section>
    }
}
