# HTML5 Standards

## Overview

All generated website pages must be valid HTML5 and follow semantic markup conventions.

## Required Elements Per Page

Every page must contain:
- `<!DOCTYPE html>` declaration
- `<html lang="en">` (or appropriate language code) at the root
- `<head>` with `<meta charset="utf-8">` and `<title>` element
- `<body>` with semantic structure: `<nav>`, `<main>`, `<footer>`

## Semantic Markup Rules

- Use `<nav>` for site navigation (primary and secondary).
- Use `<main>` for the page's primary content (one per page).
- Use `<article>` for self-contained content sections.
- Use `<section>` for thematic groupings with headings.
- Use `<aside>` for supplementary content.
- Use `<header>` and `<footer>` for section headers and footers.
- Heading hierarchy must not skip levels (h1 → h2 → h3, never h1 → h3).

## Page Title Requirements

- Every page must have a `<title>` element.
- Format: `<Page Name> — UOR Foundation`
- Maximum title length: 70 characters.

## Validation

Pages are validated with `html5ever` (the parser used by Servo). Structural checks:
- Parses without fatal errors
- `<title>` present
- `<nav>`, `<main>`, `<footer>` present

## References

- [HTML5 W3C Specification](https://html.spec.whatwg.org/)
- [HTML5 Sectioning Elements](https://html.spec.whatwg.org/multipage/sections.html)
- [html5ever Crate](https://docs.rs/html5ever/)
