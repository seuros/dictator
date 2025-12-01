---
title: [unclosed array
description: This has broken YAML syntax
pubDate: 2025-12-01
---

# Invalid YAML Frontmatter

This file has intentionally broken YAML in the frontmatter.
The `title` field has an unclosed array bracket.

This should trigger a `decree.frontmatter/invalid-yaml` error.
