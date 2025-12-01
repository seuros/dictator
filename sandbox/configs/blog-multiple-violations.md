---
pubDate: 2025-12-01
description: This has multiple violations
title: Multiple Frontmatter Violations
---

# Multiple Violations

This file has multiple frontmatter violations per .dictate.toml:

1. **Missing required field**: No `slug` field
2. **Wrong order**: `pubDate` comes before `title`
3. **Wrong order**: `description` comes before `title`

Expected errors:
- `decree.frontmatter/missing-required-field: Missing required field: slug`
- `decree.frontmatter/field-order: Field 'title' should come before 'pubDate'`
