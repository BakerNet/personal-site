use leptos::prelude::*;
use leptos_meta::Title;

use super::avatar::{Avatar, InfoBlock};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="About Me" />
        <div class="max-w-6xl mx-auto">
            <section class="flex flex-col lg:flex-row justify-center items-center lg:mt-8 xl:mt-16">
                <div class="mt-4 mr-4">
                    <Avatar />
                </div>
                <div class="max-w-full mt-4">
                    <InfoBlock />
                </div>
            </section>
            <h1 class="text-2xl my-8 text-center font-bold">
                "Hi, I'm Hans - welcome to my personal site."
            </h1>
            <section class="flex grow justify-start">
                <div class="w-full max-w-2xl">
                    <h2 class="text-xl font-bold my-8">"As a professional"</h2>
                    <p class="text-base mb-2">
                        "I've been working in software development since 2016.  I've been all the way up the career ladder from Junior Software Engineer to Senior Software Engineer to Engineering Manager to Director of Engineering and now Principal Architect."
                    </p>
                    <p class="text-base mb-2">
                        "I take a lot of pride in my work and always strive for excellence.  I have respect for my coworkers - my goal is always to lift up those around me.  I have respect for my users - I bring a customer-centric mindset into my work and go beyond specs to build the best experience for the user."
                    </p>
                    <p class="text-base mb-2">
                        "For the past 7 years, I've been working at Multi Media LLC.  We run one of the largest sites on the internet, with over 2 billion monthly sessions and petabtye scale data.  Our platform is a highly interactive live streaming web app."
                    </p>
                    <p class="text-base mb-2">
                        "I've worked all across the stack as an engineer - frontend, backend, microservices, and tooling.  In the director role, I led the team through consistent user and revenue growth, maintained business alignment, and implemented meaningful processs improvements.  I'm very proud of what I've accomplished so far at Multi Media LLC and can't wait to see what the future holds."
                    </p>
                    <p class="text-base mb-2">
                        "Try typing "<code>"cd /cv"</code> " in the header to see my resume."
                    </p>
                </div>
            </section>
            <section class="flex grow justify-end">
                <div class="w-full max-w-2xl">
                    <h2 class="text-xl font-bold my-8">"Outside of work"</h2>
                    <p class="text-base mb-2">
                        "I'm a husband and a father of an amazing daughter.  We also have 2 cats and dog who we spoil.  My wife and I love to try new foods - we're always looking for interesting places to eat when we travel."
                    </p>
                    <p class="text-base mb-2">
                        "I'm a massive nerd - some of my favorite activies with friends include Gloomhaven/Frosthaven, Magic: The Gathering, or playing DotA 2 over Discord.  But I also enjoy outdoor activies - board sports, cycling, and rock climbing."
                    </p>
                    <p class="text-base mb-2">
                        "I truly consider myself a lifelong learner.  There's nothing more enjoyable than learning something new or "
                        <strong>"even better"</strong> " improving at something."
                    </p>
                    <p class="text-base mb-2">
                        "This means I have had a "<em>"LOT"</em>" of hobbies over the years."
                    </p>
                    <p class="text-base mb-2">
                        "To list a few:  Legos, comic books, yoyo, rubik's cube, juggling, skateboarding, rock climbing, 3d printing, fixie cycling, video games (modern & retro), board games, card games, model kits (gunpla), cooking, coffee (espresso), electronics, keyboards, programming, minesweeper (try typing "
                        <code>"mines"</code> " in the header), sudoku, etc, etc, etc."
                    </p>
                    <p class="text-base mb-2">
                        "Some random facts about me:" <ul>
                            <li>
                                "In high school, I trained with the USA Olympic Bobsledding team but decided not to pursue the sport because I would have had to move across the country (Lake Placid, NY)."
                            </li>
                            <li>
                                "In college I was sponsored for yoyoing and competed in the World Yoyo Contest."
                            </li>
                            <li>
                                "In undergrad university, I was Pre-Med and only decided to switch to switch to computer science after graduation while studying for the MCAT."
                            </li>
                        </ul>
                    </p>
                </div>
            </section>
            <section class="flex justify-center items-center mt-4">
                <div class="w-full max-w-2xl text-center">
                    <h3 class="text-xl font-bold my-8">"Contact Info"</h3>
                    <p>
                        "Feel free to reach out to me for inquiries, mentorship, or opportunities."
                    </p>
                    <p>"I'm always happy to chat."</p>
                    <p>
                        "Email: "
                        <button
                            class="text-brightWhite"
                            onclick="navigator.clipboard.writeText('contact@hansbaker.com').then(() => alert('copied: contact@hansbaker.com'))"
                        >
                            <span class="font-bold">"contact@hansbaker.com"</span>
                        </button>
                    </p>
                </div>
            </section>
        </div>
    }
}
