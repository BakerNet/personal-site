---
title: "Claude Code is Eroding my AI Skepticism"
description: "A realistic look at using Claude Code for real work on existing projects"
author: Hans Baker
date: "2025-08-04T22:00:00Z"
tags:
    - ai
    - programming
---

# Claude Code is Eroding my AI Skepticism

**A realistic look at using Claude Code for real work on existing projects**

If you find yourself dismissing the recent agentic coding tools as just hype and you hate the concept of [Vibe Code](https://x.com/karpathy/status/1840909028703150286) - I get it and this article is for you.  I would hope to convince you to give these tools a real shot.  I will present a mental framework to use AI coding agents in a way that avoids frustration.

## TL;DR

- AI coding agents like [Claude Code](https://claude.ai/code) have evolved beyond hype to become genuinely useful tools
- Expect productivity gains on the order of 2x (not 10x) when used properly with realistic expectations
- Treat agents like fast-typing interns: they produce decent first drafts that need human refinement
- Plan mode is the killer feature - use it to course-correct before code generation begins
- Context is king, but avoid context pollution - be concise and targeted
- Not for every task - use for complex refactorings and boilerplate, not simple edits

---

## Background

I was a pretty strong AI skeptic and a late adopter of LLM tools for software development.

But for the past few months, I've been using Claude Code for both work and play; the productivity gains are undeniable, and results are pretty consistent.  No, you cannot Vibe Code  _real_ software, but that doesn't mean agentic coding tools aren't good enough to utilize for real work.


### Copilot and ChatGPT

Like everyone else, I tried Github Copilot and ChatGPT when they came out.

Copilot felt really interesting at first, but over time became clear it was actually a net negative on my productivity, as I would pause for a mixed result instead of just continuing to type for a fixed correct result.

ChatGPT was just bad at the use case I would try to use it for, and is a horrible developer experience.  I would try to consult ChatGPT for answers I couldn't easily google, and ChatGPT would confidently feed hallucinations for these types of questions upwards of 50% of the time.  This ended up losing me many more hours than I gained.  Additionally, the workflow of copy/pasting between chat and code is just bad.  It was clear to me very early on that Chat would not be the interface for good LLM-backed tooling.

When o1 came out, it showed potential.  I was working on my minesweeper app's analysis algorithm at the time and figured I'd see how o1 would do when tasked with creating a similar algorithm.  The results were interesting - it had no problem implementing a solution using a SAT solver (probably because that's a common approach).  But my algorithm is a sort of sliding window heuristic algorithm which is much less compute and resource intensive.  When I tried to push o1 to create a similar algorithm it failed... and again, in a confidently wrong way.

### LLM Improvements

When Gemini 2.5 Pro became available in early access, I tried it out in aistudio.  It was really good.  With Gemini, the hallucination rates were down to ~10% from the >40% I was experiencing for similar questions to ChatGPT in the past.

The LLMs were becoming more capable and reliable with the combination of reasoning models and the ability to ground the results with search.

My improved success with Gemini convinced me to try other LLM-based tools.  I started testing out Cusor for the agent mode and had some promising (but mixed) results.  The agent was able to handle a fairly complex refactoring pretty well... 

...except for deleting half of my main function.  

While Cursor's agent mode was not perfect, I was able to fit extra development into my work week that I wouldn't have had time for due my other responsibilties.  I couldn't deny the boost in productivity I gained using the tool.

## Claude Code

After seeing the potential in Cursor agent mode, I started to look for a tool that would better fit my workflows and hopefully have more reliable results.  I tried all of the CLI agents I could get my hands on at work to test and compare.  I tried early version of OpenCode, Codex CLI, and Claude Code.  Claude Code really stood out for it's polish and UX.

Claude Code fit perfectly into my workflows.  Being a Tmux + Neovim user, having Claude Code always available alongside my editor and shell is great. When using Claude Code for programming tasks, it has been _much_ more reliable than Cursor was.  It is also much slower, but I will take reliability over speed any day when it comes to generating code.

In my opinion, the killer feature that Claude Code introduced was **plan mode** - where you can easily `Shift + Tab` into a mode where Claude will output a technical plan of action, rather than immediately start producing code.  This is really the secret sauce to improving reliability of coding agents.  This mode gives you the ability to course correct before the LLM gets started, and clarify ambiguous details of your prompt.

Like with Cursor, I couldn't deny the boost in productivity I saw using Claude Code - but in addition to that, I was really enjoying the process of working with Claude Code.  I never would have thought using an LLM to generate code would be satisfying to me.  I love coding, and fully passing the act of coding off to a machine would be disappointing.  

But my workflows with Claude Code still very much keep the human-in-the-loop.  I provide a guiding hand for Claude Code to generate my ideas, utilizing my architecture, just with a lot less typing.  The final result that Claude produces is almost never code I would ship as is, but it is a decent first draft produced in a shorter amount of time.  

From the code Claude generates, I can manually edit, fix, and refactor to reach a final solution I am happy with.  All this is to say, I do not vibe code with Claude, nor do I advocate for it.

## Learnings

In my journey into agentic coding assistants, I've learned a lot along the way.  Here are a few:

The **backends** (models themselves) **are stateless**.  This is very important to keep in mind. Anything not in the context window sent as input to the LLM will not be considered when the model produces output.  Because of this, agents do not learn automatically.  

If you do not memorize an instruction in your context document, the agent will not remember that you told it to "always use testing (std lib) over testify", for example.  I assume Anthropic and others are working into building automatic memory into the software stack, but as of now, memory has to be manually curated.  In the meantime, Claude provides a convenient `#` command to append to the project context document.

There's a saying that "Context is King" and this is very true.  However at the same time **context pollution is very real**.  You don't want to tell the agent everything it could possibly need to know to complete tasks.  Every piece of context reduces the likelihood of the most important pieces of context influncing the final result.  Because of this, the context documents that are fed into the LLM need to pretty concise.

Having developer documentation referencable in the codebase is very helpful for easily providing the right context for the right task.  Including this information can even be automated by instructing the agent to refer to different docs when working on different areas of the codebase.  For example, in one project, I instructed the agent to refer to my `styleguide/testing.md` whenever working on unit tests.  This way, the agent uses the library and naming conventions I like without having to correct the output or re-prompt frequently.

Claude Code is very good at planning - even better than it is at implementing it's own plans.  When working on really large projects, I've used Claude Code to generate a technical planning document as the key deliverable and have been quite impressed with the results.  If instructed, it will accurately identify which tasks can be completed in parallel.  It will break down projects into phases / milestones which generally make sense.  It can identify tasks which are good candidates for junior engineers to take on.  These plans are incredibly helpful when working on projects which cannot be completed in a single go.

## Recommendations

There are a couple mindset recommendations I want to impart on you in order to get the most of Claude Code and other AI coding agents.

**Set realistic expectations**.  Do not try to vibe code for real software.  The AI agents still often do not produce well-architected code.  The code will often not handle edge cases, it will be missing permissions checks, and will have unnecessarily nested logic.  The agent utilizes tool calls to try to get as much relevant context as it can, but it will miss things.

**Treat the agent like an intern** who can research and type really quickly, but who lacks experience. LLM-backed agents are pretty good at following directions, but you can't expect them to be masters of your domain nor embody the wisdom built over years of work in the industry.

Treat the output of the agent as a **decent first draft, not a ready-to-go PR**.  If you are anything like me, the hardest part of programming is writing the first dozen lines of code - but having something "on paper" to overcome this coder's block has been a boon on my productivity.  Even if Claude outputs a solution I don't like, I at least have a starting point from which I can reshape the diff into a solution I _do_ like.  I am very happy with this approach of generate > edit or generate > refactor in order to reach a solution I am proud to commit.

Don't use the agent for every task.  If you know exactly the change that needs to be made to exactly which files, you will be able to make that change much faster than the agent will.  Don't fall into the trap of giving miniscule tasks to the agent out of habit otherwise you risk offsetting the productivity gains you can get on the larger tasks with the overhead it adds to smaller ones.

Keep an open mind and **experiment**.  There is a lot of nuance with these models which you can pick up through experience, not all of which is intuitive.  Anthropic does a good job with their documentation to expose some of the unintuitive nuance ([for example](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/claude-4-best-practices#example-formatting-preferences)) but there's more you will pick up on through experimentation and reflection.  It's probably not a great idea to use a high-priority, time-sensitive task as your first project for Claude Code.  Pick a time when you can afford to push through a learning hump.

## Conclusion

AI coding agents have given me the ability to fit more tasks into my busy work schedule than I would have otherwise.  Where previous LLM-based solutions were barely worth using in my opinion, Claude Code has proven to be incredibly useful.

There's a lot of hype flying around about agenting coding tools.  While the evangelists will massively oversell what the agents are capable of, I do strongly believe most software engineers _can_ benefit to some degree from tools like Claude Code.  Unless you are pretty bad at writing software and can't touch-type you will probably not 10x your productivity, but 2x is reasonably achievable.

The agentic coding tools are a massive leap ahead in usefulness compared to autocomplete and chat interfaces.  If you are an AI skeptic for software development like I was, I'd encourage you to consider trying out the newer agentic coding tools.  If you can keep an open mind and set realistic expectations, I believe you will be pleasantly surprised.
