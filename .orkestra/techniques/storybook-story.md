---
title: Storybook Story
description: >
  Write Storybook stories covering each visual state (default, loading, error, empty) for new
  or changed UI components. Follows project conventions for provider setup and file placement.
---

You are writing Storybook stories for a UI component that was added or changed in this Trak.

## Before Writing

Load the `/storybook` skill. It contains the authoritative provider setup, file conventions, and screenshot workflow for this project. Follow those patterns exactly — do not invent alternatives.

## Coverage Requirement

Write one story per meaningful visual state. At minimum, cover:

- **Default** — the component in its normal, populated state
- **Loading** — if the component has an async loading state
- **Error** — if the component can display an error condition
- **Empty** — if the component renders differently with no data

Only cover states the component actually supports. Don't fabricate states that don't exist.

## File Placement

Stories live alongside components. If the component is at `src/components/Foo/Foo.tsx`, the story file is `src/components/Foo/Foo.stories.tsx`.

## Screenshots

After writing stories, run the screenshot workflow from the `/storybook` skill and register each screenshot as a resource in your artifact output. This gives the reviewer a visual confirmation of what was built.

## Rules

- Every new or visually-changed component needs at least one story
- Follow the provider setup from the `/storybook` skill exactly
- Do not modify component implementation to make stories work — if the component is hard to story, note it in your summary
