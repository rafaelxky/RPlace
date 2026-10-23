# RPlace Completion Provider

VS Code completions for RPlace template directives and variable references.

## Features

- Suggests directives such as `def`, `place`, `for`, `match`, `include`, and `end` after an RPlace comment marker.
- Offers contextual completions for `where`, `in`, iteration sources, and match arms.
- Completes `$#` template variables and `$g#` global variables in template text.
- Supports `//-`, `/*-`, and `*///-` directive markers, including RPlace templates embedded in other file types.

The provider activates in all text documents, but only returns suggestions in recognized RPlace contexts.
