---
title: "The Best Feeling in Software Engineering is Going Away"
description: "A rant about how use of LLMs removes one of the most rewarding parts of programming"
author: Hans Baker
date: "2026-04-30T05:00:00Z"
tags:
    - ai
    - programming
---

# The Best Feeling in Software Engineering is Going Away

In my last blog post ([claude_code.md](/blog/claude_code)), I was in the honeymoon phase of working with LLMs for development.  The tools were finally in place to make using the technology practical and seeing real gains in productivity.  I still stand by that post today, but my relationship to the tools has certainly shifted.



Today, I still use Claude Code at work **reluctantly**.  The agentic coding tools are still unquestionably (in my opinion) able to give a positive boost in productivity - and as an employee, there's an implicit contract with the employer that one maximizes productivity within the bounds of working hours.  I also do love using coding agents for small personal software when I don't care about learning the underlying tech.

So, why **reluctantly**?

## A Recent Anecdote

I recently got access to the [Jai](https://www.youtube.com/watch?v=TH9VCN6UkyQ) compiler and have been having fun learning the language.  Prior to that, I started working on my first video game in [Odin](https://odin-lang.org/) and decided I wanted to rewrite it in Jai.

Last week when I was driving home from [NAB Show](https://www.nab.org/documents/newsroom/pressRelease.asp?id=7436), I stopped at a charging station to charge my car.  I was sitting in my car and decided to pull out my personal laptop to do some programming.   Since I was programming on my laptop without WiFi, I was relying solely on Jai module source code and the knowledge I gained from doing this once in Odin before to make progress.

I was working on some MacOS platform layer code and ran into a `SIGSEGV`.  I was struggling to debug it and after a short troubleshooting session, I gave in and used Claude on my cellphone to attempt to get a quick solution.  Claude proceeded to confidently point out imaginary issues, lead me down pointless rabbit holes, and suggest that rebuilding Metal bindings was a "smart" idea.  I could rant more about Claude here, but that would be besides the point.  When I realized the conversation with Claude was not getting me anywhere, I went back to old methods.  I used `lldb` to step through and examine memory leading up to the fault.  Sure enough, there was an attempt to dereference a null pointer and I was able to deduce enough to know that the null pointer in question was a selector being sent to `objc_msgSend`.  From here, it was obvious that I missed an initialization function.  When I finally got the code to run and my MacOS window popped up instead of a Segmentation fault I got a rush of dopamine and made a verbal "fuck yeah."

Overcoming this point of friction on my own after the bot had failed to hand me the answer was the **best feeling** I've had programming in months.  The reality is that Claude Code could probably have gotten to the same solution probably a bit faster than I did with some nudging, but I wouldn't have experienced an ounce of accomplishment.

It turns out solving problems instead of having the solution handed to you on a silver platter is rewarding (duh).

## How I'm Moving Forward

Even before this event, I was distancing myself from LLM coding agents for my personal project work.  My goal is to _learn_ how to make games from scratch, not to watch a bot make one after all.  I was still using Claude with a `CLAUDE.md ` which prompted it to act as a tutor and progress tracker - and I would ask it to quiz me on what I had learned as I progressed through the early stages of bootstrapping a simple game engine.  But even with this approach, I've found that I would fall back to asking the LLM for answers when I got stuck instead of pushing through on my own.  Sure, I would have Claude quiz me to help solidify the knowledge - so I was definitely learning things - but it still wasn't satisfying.

I've gotten a hit of the dopamine that comes from pushing through friction.  The knowledge I gained is now burned into my head and I'll be able to spot similar issues quickly in the future.  I became just a little bit better as a programmer in the process.  I want that experience again - so my plan is to shrug off the urge to ask for answers.  My fingers will type `lldb` instead of `claude` as I push my way through.

The unfortunate thing is, I can't make the commitment to do this at work.  I'll still get to experience the best feeling in **programming** in my personal projects when I get time to work on them.  But this blog post is titled "the best feeling in **Software Engineering** is going away" because the workplace demands bypassing friction instead of pushing through it.  I think this lack of feeling of accomplishment when using coding agents was bringing me pretty close to burnout (or at least contributing a bit).

I wonder how many others have lost some joy in their work in the name of productivity?
