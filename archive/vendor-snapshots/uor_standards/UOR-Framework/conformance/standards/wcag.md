# WCAG 2.1 AA Standards

## Overview

All website pages must meet WCAG 2.1 Level AA requirements for accessibility.
The conformance suite automates checks for machine-verifiable criteria.

## Automated Checks (Implemented)

| Criterion | WCAG SC | Check |
|-----------|---------|-------|
| Images have text alternatives | 1.1.1 | Every `<img>` has a non-empty `alt` attribute |
| Language of page | 3.1.1 | `<html>` has `lang` attribute |
| Link purpose (in context) | 2.4.4 | No empty `<a>` elements without `title` or `aria-label` |

## Manual Checks (Not Automated)

These require human review:
- **Color contrast** (1.4.3): Text contrast ratio ≥ 4.5:1 (normal text) or 3:1 (large text)
- **Keyboard navigation** (2.1.1): All interactive elements reachable via keyboard
- **Focus visible** (2.4.7): Focus indicator is visible
- **Consistent navigation** (3.2.3): Navigation consistent across pages
- **Error identification** (3.3.1): Form errors clearly identified (where applicable)
- **Resize text** (1.4.4): Text resizable to 200% without loss of content

## CSS Requirements

- Text must use relative units (`rem`, `em`, `%`) to support user text scaling.
- No `display: none` on focusable elements without an accessible alternative.
- `:focus` styles must be visible and not `outline: none` without replacement.

## Skip Navigation

Pages with navigation blocks must include a skip-to-content link:

```html
<a href="#main-content" class="skip-link">Skip to main content</a>
```

## Form Labels

All form inputs must have associated `<label>` elements:
```html
<label for="search-input">Search</label>
<input type="search" id="search-input" name="q">
```

## References

- [WCAG 2.1 W3C Specification](https://www.w3.org/TR/WCAG21/)
- [WCAG 2.1 Quick Reference](https://www.w3.org/WAI/WCAG21/quickref/)
- [Understanding WCAG 2.1](https://www.w3.org/WAI/WCAG21/Understanding/)
