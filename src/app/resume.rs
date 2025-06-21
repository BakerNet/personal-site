use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn CVPage() -> impl IntoView {
    view! {
        <Title text="CV / Resume" />
        <div class="grid mx-auto">
            <h1 class="font-bold text-2xl text-center mb-8">
                "CV / Resume"
                <a
                    href="/HansBakerResume.pdf"
                    download="HansBakerResume.pdf"
                    class="relative top-1 ml-4 text-white"
                >
                    <i class="extra-download" />
                </a>
            </h1>
            <Resume />
        </div>
    }
}

#[component]
fn Resume() -> impl IntoView {
    view! {
        <div
            id="resume"
            class="grid max-w-4xl min-h-[inherit] grid-cols-1 md:grid-cols-3 p-8 bg-[#FFFFFF] text-background leading-snug shadow-2xl rounded-lg border border-muted/20"
        >
            <Sidebar />
            <Experience />
        </div>
    }
}

#[component]
fn Sidebar() -> impl IntoView {
    view! {
        <div class="space-y-4 p-2">
            <div class="flex flex-col space-y-4 text-center">
                <div class="space-y-4">
                    <div>
                        <h2 class="text-2xl font-bold">"Hans Baker"</h2>
                    </div>
                    <div class="flex flex-col items-center rounded-sm border border-primary px-3 py-4 text-sm">
                        <div class="flex flex-col items-start gap-y-1.5 text-left">
                            <div class="flex items-center gap-x-1.5">
                                <i class="extra-location"></i>
                                <div>"San Diego, CA"</div>
                            </div>
                            <div class="flex items-center gap-x-1.5">
                                <i class="extra-email"></i>
                                <a
                                    href="mailto:hansbaker90@gmail.com"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    "hansbaker90@gmail.com"
                                </a>
                            </div>
                            <div class="flex items-center gap-x-1.5">
                                <i class="extra-link"></i>
                                <a
                                    href="https://hansbaker.com"
                                    target="_blank"
                                    rel="noreferrer noopener nofollow"
                                    class="inline-block"
                                >
                                    "https://hansbaker.com"
                                </a>
                            </div>
                            <div class="flex items-center gap-x-1.5">
                                <i class="devicon-github-plain" />
                                <a
                                    href="https://github.com/BakerNet"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    "/BakerNet"
                                </a>
                            </div>
                            <div class="flex items-center gap-x-1.5">
                                <i class="devicon-linkedin-plain" />
                                <a
                                    href="https://linkedin.com/in/hansbaker"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    "/in/hansbaker"
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <section id="skills" class="grid">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Skills"</h3>
                <div class="grid gap-x-6 gap-y-3" style="grid-template-columns: repeat(1, 1fr)">
                    <div class="space-y-2">
                        <div>
                            <h4>"Frontend"</h4>
                        </div>
                        <p class="text-sm">
                            "TypeScript, JavaScript, DevTools, React, WebAssembly (WASM), Websockets, Leptos, HTML, CSS"
                        </p>
                    </div>
                    <div class="space-y-2">
                        <div>
                            <h4>"Backend"</h4>
                        </div>
                        <p class="text-sm">
                            "Python, Go, Rust, SQL, PostgreSQL, Redis, Django, Celery, Axum, Sqlite"
                        </p>
                    </div>
                    <div class="space-y-2">
                        <div>
                            <h4>"DevOps"</h4>
                        </div>
                        <p class="text-sm">
                            "Docker, Terraform, Cloudflare, CodeQL, NewRelic, GCP, AWS, Linux"
                        </p>
                    </div>
                    <div class="space-y-2">
                        <div>
                            <h4>"Tooling"</h4>
                        </div>
                        <p class="text-sm">
                            "Git, GitHub, GHA, CI, Build Systems, Webpack, ESLint, Custom/Inhouse"
                        </p>
                    </div>
                </div>
            </section>
            <section id="interests" class="grid">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Hobbies & Interests"</h3>
                <div class="grid gap-x-6 gap-y-3" style="grid-template-columns: repeat(1, 1fr)">
                    <div class="space-y-0.5">
                        <p class="text-sm">
                            "Programming, Rock Climbing, Comic Books, Games, Food, Coffee, Tinkering, Learning new things"
                        </p>
                    </div>
                </div>
            </section>
            <section id="education" class="grid">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Education"</h3>
                <div class="grid gap-x-6 gap-y-3" style="grid-template-columns: repeat(1, 1fr)">
                    <div class="space-y-2">
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"CSU, Sacramento"</strong>
                                    <div>"Philosophy & Pre-Med (B.A.)"</div>
                                    <div>"3.8 GPA"</div>
                                    <div class="font-bold">"2009-2014"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="space-y-2">
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"UCSD, SDSU, and SDCCD"</strong>
                                    <div>"Computer Science (non-Degree)"</div>
                                    <div>"3.9 GPA"</div>
                                    <div class="font-bold">"2014-2017"</div>
                                </div>
                            </div>
                            <p>"Completed all pre-requisites for M.Sc. program"</p>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn Experience() -> impl IntoView {
    view! {
        <div class="col-span-2 space-y-4 p-2">
            <section id="summary">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Summary"</h3>
                <p>
                    <strong>
                        "Innovative Software Engineer with 7+ years of experience building
                        and optimizing scalable web applications."
                    </strong>
                    " Proven track record of delivering performant software across the stack.  Building better user experiences through deep expertise in JavaScript internals, browser performance optimization, and reactive frontend development. Adept at building, integrating, and optimizing backend services with experience improving site reliability for one of the world's highest traffic websites."
                </p>
            </section>
            <section id="experience" class="grid">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Experience"</h3>
                <div class="grid gap-x-6 gap-y-3" style="grid-template-columns: repeat(1, 1fr)">
                    <div class="space-y-2">
                        <div>
                            <h4 class="text-left font-bold">"Multi Media, LLC"</h4>
                        </div>
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"Principal Architect"</strong>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"Mar 2024 - Present"</div>
                                </div>
                            </div>
                        </div>
                        <p>
                            "Return to technical work after voluntarily stepping down from leadership. Working to ensure engineering excellence within Multi Media LLC while wearing several hats. Responsibilities include architecture reviews, performance management, SRE improvements & optimizations, building tools, and consulting on feature work."
                        </p>
                        <p>
                            "Project highlight: Built a code ownership and review assignment tool in Go with a CLI and GitHub Action to run in CI. Provides flexible configuration for monoliths and monorepos."
                        </p>
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"Head of Engineering"</strong>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"Jul 2021 - Mar 2024"</div>
                                </div>
                            </div>
                        </div>
                        <p>
                            "Director-level position. Grew team of world-class software engineers. Set strategy and vision for department in alignment with company goals. Implemented various processes and developer experience improvements. Proudly maintained inclusive culture and very low turnover rate."
                        </p>
                        <p>
                            "During time as Head of Engineering,"
                            <a
                                target="_blank"
                                rel="noopener noreferrer nofollow"
                                href="https://chaturbate.com"
                            >
                                "chaturbate.com"
                            </a> "saw upward Daily & Monthly user growth for 32/33 months."
                        </p>
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"Software Engineering Manager"</strong>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"Mar 2020 - Jul 2021"</div>
                                </div>
                            </div>
                        </div>
                        <p>
                            "Managed team of Frontend software engineers. Grew team while developing new and junior members. Made various improvements to developer experience through tooling, inhouse library development, implementing CI, containerization, documentation, and adding light process."
                        </p>
                        <p>
                            "Created system for integrating React components into our inhouse TypeScript framework to allow for iterative adoption of React. Also built JSX factory to be able to use JSX syntax with our inhouse TypeScript framework."
                        </p>
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"Software Engineer"</strong>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"Dec 2017 - Mar 2020"</div>
                                </div>
                            </div>
                        </div>
                        <p>
                            "Worked full-stack on our primary web application (Python, Django, TypeScript / JavaScript, HTML, CSS, PostgreSQL / TimescaleDB, Redis) as well as built microservices (Go / Golang). After several successful frontend projects, was asked to review all frontend changes for the entire engineering team. Implemented several DX improvements for faster iteration and better testability."
                        </p>
                        <p>
                            "Project highlight: Built endless scroll feed of user-uploaded content. Backend was distributed microservice in Go. Frontend was built in TypeScript and supported all browsers back do IE8. Was live in production with 5-60k concurrent active users."
                        </p>
                    </div>
                    <div class="space-y-2">
                        <div>
                            <h4 class="text-left font-bold">"Bayside Networks"</h4>
                        </div>
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <strong>"IT Consultant & Full-stack Developer"</strong>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"Dec 2015 - Dec 2017"</div>
                                </div>
                            </div>
                        </div>
                        <p>
                            "Split time between System Administration / IT Consultancy (primarily Windows server) and Full-stack development (Linux / Apache / PHP / MySQL). Full-stack development work was extending a highly customized CRM software solution in LAMP stack. Developed several new systems including a project management interfaces and a business client portal."
                        </p>
                    </div>
                </div>
            </section>
            <section id="projects" class="grid">
                <h3 class="mb-2 border-b pb-0.5 font-bold">"Personal Projects"</h3>
                <div class="grid gap-x-6 gap-y-3" style="grid-template-columns: repeat(1, 1fr)">
                    <div class="space-y-2">
                        <div>
                            <div class="flex items-start justify-between">
                                <div class="text-left">
                                    <div class="font-bold">"Minesweeper Webapp"</div>
                                    <div>"Fullstack Rust Minesweeper Web App"</div>
                                </div>
                                <div class="shrink-0 text-right">
                                    <div class="font-bold">"2023-2024"</div>
                                </div>
                            </div>
                            <div class="flex items-center gap-x-1.5">
                                <i class="extra-link"></i>
                                <a
                                    href="https://mines.hansbaker.com"
                                    target="_blank"
                                    rel="noreferrer noopener nofollow"
                                    class="inline-block"
                                >
                                    "https://mines.hansbaker.com"
                                </a>
                            </div>
                        </div>
                        <div>
                            <p>
                                "Minesweeper client with backend game engine, multiplayer support, login via OAuth2, replays w/ analysis, and personal statistics. Inspired by "
                                <a
                                    target="_blank"
                                    rel="noopener noreferrer nofollow"
                                    href="http://monkeytype.com"
                                >
                                    "monkeytype.com"
                                </a> ", and my addiction to logic-based games."
                            </p>
                            <p>
                                "Built with Rust, Leptos, Axum, and TailwindCSS. Containerized, deployed to Fly.io, and protected by Cloudflare."
                            </p>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}
