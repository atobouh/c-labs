# Machine Room — Course studio proposal

## Product model

An offline learning studio, rather than a debugger with lessons attached. A local course collection contains chapters, original explanations, custom diagram exercises, guided labs, and a substantial project per chapter. Passing that project's rubric unlocks the next chapter. A shared lexicon connects concepts across courses. An open lab supports independent C work and locally configured external libraries.

This phase builds the shell and reviewable interactions. It does not populate the complete course database or implement the compiler runner.

## Visual direction

- Canvas: `#F6F6F3`, a warm neutral light studio surface.
- Paper: `#FFFFFF`, for reading and diagrams.
- Ink: `#202632`, for primary text.
- Secondary ink: `#616D80`, for supporting text with readable contrast.
- Course accent: `#3D5B93`, used for selection and the next action.
- Diagram accent: `#397A76`, used to distinguish memory objects from addresses.

Instrument Sans provides the locally bundled interface and reading typography; a system monospace face handles source code. Use 15–16px reading text, 13px supporting UI, and 32–36px page titles. Avoid decorative all-caps labels and miniature metadata.

## Layout

One persistent sidebar provides Courses, Projects, Lexicon, Open lab, and Review. The current chapter appears as a quiet continuation link. Course progression belongs inside the course, not in another global navigation system.

```text
Sidebar     | Course collection / continue chapter
            | Course overview / chapters and project gates
            | Chapter: Learn → Exercises → Labs → Project
            | Reading or diagram canvas + relevant context
            | Open lab: files + source + console / local libraries
```

The signature visual is a custom memory diagram integrated into the lesson, with concrete cell labels and interactive selection. Courses use subject-specific diagrams rather than imitation book covers. A project is presented as a work brief with deliverables and checks, not as another lesson tile. The current proposal is `studio.html`; earlier workbench explorations are retained separately.

## Principles

Clarity comes from typography and placement before borders or badges. Controls appear where they are needed. No animated backgrounds, score dashboards, fake terminal activity, or redundant sidebars. Keep genuine teaching content and UI research imagery separate. The supplied Octet screenshot calibrates craft and finish; its bookshelf layout, visual identity, and content are not copied.

## Content database plan

First inventory sources and their license terms. Create original course records and lexicon entries with source citations, prerequisites, learning objectives, diagram descriptions, lab fixtures, and project rubrics. Review each factual claim against primary references. Compile and run shipped examples and checks under declared C modes. Version course packs, bundle permitted resources with attribution, and store the catalog and progress locally. Full-text search should work without network access.
