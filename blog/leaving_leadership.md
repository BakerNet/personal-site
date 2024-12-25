---
title: Why I Stepped Down from Leadership
description: Reflections on my return to engineering.  Discussing why I stepped down from a director-level leadership position to return to a more technical role.
author: Hans Baker
date: "2024-11-30T12:00:00Z"
tags:
    - work
---

# Why I Stepped Down from Leadership {#leaving-leadership}

**Reflections on my return to engineering**

Near the end of 2023, I made the decision to return to individual contribution work after 2.5 years as Head of Engineering at [Multi Media LLC](https://multimediallc.com).  In this post, I share my leadership journey and describe my reasons for stepping down.

If you are short on time, skip to [Factors for my Decision](#factors-for-my-decision) for a summary.

## Background {#background}

I started my path into computer science and later software engineering because of my love for [formal logic](https://en.wikipedia.org/wiki/Logic#Formal_logic) (more on that in a future blog post).  Computer science, to me, felt like the perfect extension: applied logic in action.  I became a software engineer not for the money — though that’s a nice bonus - nor solely for the products I build, but for the joy of exercising applied logic.

I have spent my entire tech career at Multi Media LLC, climbing the ladder from Junior Engineer to Head of Engineering.  Despite Multi Media LLC's significant revenue and scale, the company has remained relatively small in size, providing unique opportunities to have impact.  There were less than 10 engineers when I joined and the engineering department is still well less than 100 in all roles.

I take a pride in delivering my best work regardless of role, an attitude which has contributed to multiple promotions.  Additionally, I love to try new things.  Multi Media LLC hired me early in my career for my expertise with Go, a language I had embraced before it gained widespread popularity.  Early on, I specialized in highly concurrent servers and distributed systems, though I didn't shy away from full-stack development. 

I ended up having a very successful project built using our in-house TypeScript framework.  After successfully delivering this project, I was brought in to rescue a rewrite of our most interactive page which had gone underwater.  What had been stuck in QA for months, I was able to get out the door in a few weeks.

This success lead the Head of Engineering to entrust me with reviewing _all_ frontend changes company-wide.  I tried my best to elevate the quality of work the team was contributing but made sure to always be respectful and professional.  When the Head of Engineering decided to break up the team into a Frontend Team and a Backend Team, I volunteered to be the Engineering Manager of the Frontend Team.  

I really enjoyed being a manager.  Seeing the tangible impact of my mentorship and my team's progress was incredibly fulfilling.

A year and a half later, the CEO reached out to me to ask if I was interested in taking over as the Head of Engineering.  I gladly accepted the position, ready to take on new challenges and have wider impact.  I had many ideas for how I could improve our engineering processes.

There was no doubt in my mind I would enjoy the role.

## Head of Engineering Role {#head-of-engineering-role}

My journey as Head of Engineering was a dynamic path of continuous growth and learning. To make sense of it, I’ve divided this period into distinct 'eras,' each marked by its own challenges, milestones, and lessons.

### Early Days {#early-days}

When I first took over as the Head of Engineering at Multi Media LLC, we had a Matrix organization structure where Software Engineers were directly reporting to PMs for day-to-day work and management, but they also all indirectly reported to me.  This meant I had a couple dozen "direct" reports, but management did not take a large amount of my time.

During this period, I had a great impact on developer experience and processes.  We moved to a containerized development environment, which improved onboarding by orders of magnitude.  I worked on improving other aspects of our onboarding process.  I made some meaningful improvements to developer experience on the frontend as well.  I started meeting with our QA leaders to address issues and improve efficiency.  I also of course performed 1-on-1's with engineers to support their growth and development.

When I took over, I immediately tackled some outdated policies.  Some examples:
- We had a policy of not adding code comments
- Engineers were required to set styles for TypeScript components directly using DOM API instead of CSS/SCSS
- We supported all browsers back to IE8
  - This required many polyfills and avoiding modern CSS but provided no revenue

Changes to policies like these had a meaningful positive impact on productivity.

We had not invested in upgrading our dependencies in years.  We were still on Python 2 our version of Django was 2 years past LTS.  I ensured we had resources to bring our tech stack into the modern age.  I directly contributed to these efforts on the frontend.

At the same time, I was continuing to review most frontend code.

This might not align with traditional Head of Engineering responsibilities - and to be frank - it probably shouldn't.  Despite this, I found great satisfaction during this time as Head of Engineering.  I stayed involved in technical work, directly improved the experience of developers, and witnessed our team becoming more productive.

### Recognizing Leadership Gaps {#recognizing-leadership-gaps}

In hindsight, I led the team with a certain naïveté. I created an environment where I personally would have thrived but overlooked critical aspects of leadership, such as performance management.  As a result, some underperforming engineers remained unnoticed for longer than they should have.  Promotions were handled ad hoc, without a consistent or transparent framework to ensure fairness and readiness.  This led to some promotions happening prematurely, before individuals were fully prepared for their new roles.

I didn't _really_ grow into being a true leader because the demands of the role hadn't required it.  Eventually, the CEO and I discussed reorganizing into functional teams, formalizing hiring and performance management practices, and ultimately maturing the organization.

This was the beginning of a major mindset shift for me.  I needed to evolve into a true leader, grow engineers into managers, learn to delegate, and spearhead organizational change.

My early missteps laid the foundation for the intentional leadership style I've since developed.  I committed to my growth and embraced director-level responsibilities with focus and determination.

### Becoming a Director {#becoming-a-director}

We reorganized the team so that all of the engineering department was truly reporting up to me - I wasn't just the guy who needed to sign off on promotions anymore.  I interviewed all of the engineers to identify people who had the aptitude and interest in becoming a manager.  We formed a management group and for the most part kept the existing teams of engineers together during this transition.  It was exciting to be trying something new.

Stepping into the director role required me to step back from hands-on technical work.  I could no longer afford to spend half my hours in the code.  I identified an engineer who could take over as the frontend lead and handed over the reigns.

I started taking management training courses and studying hiring practices.  I took classes on creating high performing technical teams and organizational leadership.  I was learning a lot and directly applying what I was learning.

During this time, I created a career ladder document with competency descriptions, performance metrics, development paths, growth opportunities, and a matrix for managers to use with their reports.  With real performance management in place, we could evaluate performance more objectively, and give engineers a clear path to promotion.  The Talent Acquisition team and I collaborated to overhaul our hiring process.  We created formalized rubrics to reduce interviewer bias and compare candidates fairly.

After I was introduced to the concept of [Trunk-Based Development](https://trunkbaseddevelopment.com), I spearheaded its adoption across the engineering department.  I applied my new skills to make the migration as effective and smooth as possible, and to great effect.  TBD almost entirely eliminated merge conflicts, forced better planning practices, and allowed us to deploy to production almost daily with less risk.  Upper management could also now test in-development features in production, which they greatly appreciate.

I was working on creating a vision to move forward with, rather than just going with the flow.  I was executing the company strategy and creating alignment of the engineering department with company goals.

All of these things were good and correct actions to take.  I'm very proud of how I was leading the team and the changes I was making.  **But I wasn't enjoying the work**.  I wasn't passionate about leading like I was about software engineering.

### Reckoning Phase {#reckoning-phase}

I learned that I have a deep need for frequent mental stimulation.  I have come to suspect that I have undiagnosed ADHD, which would explain that.  Coding satisfied that need quite well; leadership didn't.

I dreaded leadership courses, policy documents, and meetings - tasks that felt routine rather than mentally stimulating.  The work did not require the analytical parts of my brain which were screaming for attention.

While I knew the work I was doing was impactful and beneficial, I found myself unable to truly feel the victories.  When the engineering teams achieved success, I felt more like I was a cheerleader on the sidelines rather than an active participant.  The growing distance created by the hierarchy left me feeling detached, with little sense of accomplishment.

I started to crave technical challenges outside of work, as my role no longer provided the mental stimulation nor the rewarding feedback loop I deeply valued.  Fortunately, around this time I stumbled upon a [ThePrimeagen](https://www.youtube.com/theprimeagen) YouTube video where he was very clearly enjoying himself while coding.  It inspired me to pick programming back up as a hobby... And oh did it feel good to get back to my roots.  I taught myself Vim motions, became better at using the terminal, picked up Rust, and built projects again.

I treated hobby coding as the technical counterbalance for my barely-technical director-level job.

This kept me sane and brought back the energy I needed to get through my work responsibilities... **Until my wife and I had a baby**.

Babies and toddlers require a LOT of attention.  Especially babies like ours who (like me) needed constant mental stimulation.  I could no longer read engaging technical material, play video games, or code in my spare time.  My attention was needed to keep this bobble head from screaming, crying, or hurting herself.  The work of raising a baby, like director-level work, does not require using the analytical parts of your brain (it doesn't even allow for it).

My counterbalance was lost and it was starting to make me go crazy.

### Stepping Down {#stepping-down}

The event which finalized for me that I needed to step down was when I was asked to implement a major change I disagreed with.  I wanted to oppose the shift, but I felt like I wasn't experienced enough to really prove (or even be sure myself) that my point of view was correct.  I didn't have experience at other companies to fall back on.  There was some unhappiness with the status quo, so I couldn't justify my stance by leaning on the merits of our current situation.  It became clear to me the correct move for both myself and the company was to step down from the director position and suggest we hire someone with more experience to lead the team.

And so, after some deep thought and discussion with my family, I expressed my intent to step down from the director-level position and return to IC work.

When word got around, several colleagues reached out to me expressing how much they appreciated my time in leadership, which was really wonderful to hear.

## Factors for my Decision {#factors-for-my-decision}

A lot of compounding issues, feelings, and experiences played into my decision to step down from the director role.  Here is a compiled list of the contributing factors:

- The volume of meetings as a director was mentally exhausting
- The teachings from courses on leadership, management, and business _is_ valuable, but I don't enjoy the pseudo-scientific approaches nor am I compelled by cases studies
  - This is just the nature of business and psychology, I am not arguing that these fields are doing things wrong.  But coming form hard science background, it just doesn't feel right to me
  - I craved low-level precision as a counterbalance to the high-level concepts of my coursework
- The concrete victories that come out of your actions are at least one level removed
  - I didn't get the same feeling of satisfaction when things went well as I did doing individual contribution, direct mentorship, or direct management.
- I realized I stayed in leadership out a sense of duty to my team, rather than personal fulfillment
- I missed being in a technical role
  - I found my hobby coding to be more fulfilling than my job as director
- The responsibilities of a director are not mentally stimulating in a way that satisfied my brain's needs
  - This issue became **much** more problematic when I had a baby to raise and was no longer able to fill my free time with mentally stimulating activities
- I had philosophical disagreement with others in leadership about process
- I only have experience in one software company, so I did not feel confident in my intuitions as a leader

The above list is roughly in chronological order of how these factors came to play a part in my eventual decision to step down.

## Aside on the Peter Principle {#aside-peter-principal}

I assume there will likely be readers of this blog post that immediately think of the [Peter principle](https://en.wikipedia.org/wiki/Peter_principle); that I just rose to my level of incompetence.  This was almost certainly true at least for my first year in the role - I was not competent as a director of an engineering department.  However, I actually think I have strong aptitudes for leadership and honestly believe I handled the job well beyond my experience level.

I was learning, improving, and taking the role very seriously.  I didn't get fired from the position.  There's no doubt in my mind that a more experienced leader could have done a better job than I did in the director role.  There's also no doubt in my mind that if I had more experience with other companies before joining Multi Media LLC, I would have been more effective and more confidant in my intuitions.

Focusing on the Peter principle would miss the point of this blog post.

## Summary {#summary}

I am deeply grateful to Multi Media LLC for trusting me to lead the engineering department for nearly three years.  I'm very proud of what I accomplished as the director.  The experience was invaluable for growing as a software engineering professional and learning more about myself as a person.

When the new Head of Engineering took over earlier this year, I began transitioning back to engineering - and it felt **immediately energizing**.  Where before I was relieved to end my day, my wife started having to peel my fingers off the keyboard to come to dinner.

Leadership was a meaningful chapter in my career, but stepping down has allowed me to realign with my passion and reignite my love for engineering.

As for takeaways for you the reader - I don't write this blog post to discourage anyone from pursuing leadership.  But even those with an aptitude for leadership might find it’s not the right fit.

I hope my story inspires others to reflect on their own careers and to recognize when a change might bring greater fulfillment.
