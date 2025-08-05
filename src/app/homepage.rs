use leptos::prelude::*;
use leptos_meta::Title;

use super::avatar::{Avatar, InfoBlock};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="About Me" />
        <div class="max-w-6xl mx-auto page-content">
            <section class="flex flex-col lg:flex-row justify-center items-center gap-4 lg:gap-8 lg:mt-8 section-content">
                <div class="flex-shrink-0">
                    <Avatar />
                </div>
                <div class="max-w-full overflow-x-auto">
                    <InfoBlock />
                </div>
            </section>
            <h1 class="text-2xl my-8 text-center font-bold section-content">
                "Hi, I'm Hans - Software Engineering Leader"
            </h1>
            <section class="flex flex-col lg:flex-row gap-8 lg:gap-12 section-content">
                <div class="w-full lg:max-w-2xl">
                    <h2 class="text-xl font-bold my-8">"Professional Experience"</h2>
                    <p class="text-base mb-4 leading-relaxed">
                        "8+ years building and scaling software at a top-tier platform serving "
                        <strong>"2+ billion monthly sessions"</strong>
                        " with petabyte-scale data infrastructure."
                    </p>
                    <p class="text-base mb-4 leading-relaxed">
                        "My career journey: "
                        <span class="text-cyan">
                            "Junior Engineer → Senior Engineer → Manager → Director → Principal Architect"
                        </span>
                        ". I bring both deep technical expertise and proven leadership experience."
                    </p>
                    <p class="text-base mb-4 leading-relaxed">
                        "Core strengths: Full-stack development (Rust, Go, Python, TypeScript), system architecture, team leadership, and driving measurable business impact. As Head of Engineering, I led the team through "
                        <strong>"32 consecutive months of user and revenue growth"</strong>
                        " while maintaining engineering excellence."
                    </p>
                    <div class="bg-brightBlack/30 p-4 rounded-md border-l-4 border-purple mb-4">
                        <p class="text-sm text-purple mb-2 font-medium">
                            "💡 Always interested in exceptional opportunities"
                        </p>
                        <p class="text-sm">
                            "While I'm engaged in my current role, I'm always open to discussing truly compelling Staff+ Engineering positions."
                        </p>
                    </div>
                    <p class="text-base mb-2">
                        "Explore my experience: "<code>"cd /cv"</code> " or browse my musings: "
                        <code>"ls /blog"</code>
                    </p>
                </div>
                <div class="w-full lg:max-w-2xl">
                    <h2 class="text-xl font-bold my-8">"Personal & Values"</h2>
                    <p class="text-base mb-4 leading-relaxed">
                        "Family-focused professional with a passion for continuous learning and problem-solving. I believe in "
                        <strong>"empathy-driven leadership"</strong>
                        " and building inclusive, high-performing teams."
                    </p>
                    <p class="text-base mb-4 leading-relaxed">
                        "My diverse interests reflect my curiosity: competitive strategy games, rock climbing, specialty coffee, and building mechanical keyboards. This breadth helps me approach engineering challenges from unique angles."
                    </p>
                    <div class="bg-brightBlack/30 p-4 rounded-md mb-4">
                        <p class="text-sm font-medium text-green mb-2">"🎯 What drives me:"</p>
                        <ul class="text-sm space-y-1">
                            <li>"Building systems that scale and teams that thrive"</li>
                            <li>"Turning complex problems into elegant solutions"</li>
                            <li>"Mentoring engineers and fostering growth"</li>
                            <li>"Creating exceptional user experiences"</li>
                        </ul>
                    </div>
                    <p class="text-base mb-2">
                        "Fun facts: I've competed internationally in yoyoing and trained with the USA Olympic Bobsled team. I bring the same dedication to excellence in engineering."
                    </p>
                    <p class="text-base mb-2 text-muted">
                        "Try " <code>"mines"</code> " in the terminal for a quick game break!"
                    </p>
                </div>
            </section>
            <section class="flex justify-center items-center mt-8 section-content">
                <div class="w-full max-w-2xl text-center">
                    <h3 class="text-xl font-bold my-8">"Let's Connect"</h3>
                    <div class="bg-brightBlack/30 p-6 rounded-lg border border-muted/30">
                        <p class="text-lg mb-4 text-cyan font-medium">
                            "Open to collaboration and interesting conversations"
                        </p>
                        <p class="mb-4">
                            "Whether you want to discuss engineering challenges, explore potential collaborations, or share an exciting opportunity, I'd love to hear from you."
                        </p>
                        <div class="flex flex-col sm:flex-row items-center justify-center gap-4 mt-6">
                            <button
                                class="bg-cyan/20 hover:bg-cyan/30 text-cyan px-6 py-3 rounded-md font-medium transition-all duration-200 border border-cyan/30"
                                onclick="navigator.clipboard.writeText('contact@hansbaker.com').then(() => alert('📋 Email copied: contact@hansbaker.com'))"
                            >
                                "📧 contact@hansbaker.com"
                            </button>
                            <div class="flex gap-3">
                                <a
                                    href="https://linkedin.com/in/hansbaker"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="text-blue hover:text-brightBlue text-2xl"
                                    aria-label="LinkedIn Profile"
                                >
                                    <i class="devicon-linkedin-plain"></i>
                                </a>
                                <a
                                    href="https://github.com/BakerNet"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="text-white hover:text-brightWhite text-2xl"
                                    aria-label="GitHub Profile"
                                >
                                    <i class="devicon-github-plain"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}
