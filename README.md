# Rusty Api Modeler

Generates API models from openapi spec using language-agnostic templating and spec system. When ready, should support any or most of the programming languages.

Currently supports following languages out-of-box:
* Rust
* Go

However, a language can be implemented by supplying a language yaml file and some needed templates for generation. Contributions are welcome!

## Usage

Start off with a `.ramconfig` file like this:

```
# Example ram config file

# OpenAPI specification file path, currently only supports openapi 3
openapi: "./res/petstore.yaml"

# Target lang spec
# Can be file path or built-in yaml spec
lang: "rust"

# Template files, optional and used for overriding built-in templates
templates:
  # Path for custom model template file
  model: "templates/custom_model.rs.template"

# File paths, optional
# This example places models to src/models
paths:
  root: "src",
  model: "models"
```

With configuration file in-place, run `ram` in the directory to generate models from the spec.

## Templating

Supports using built-in or custom templates by configuration.

Templating uses [handlebars](https://handlebars-draft.knappi.org/guide) syntax, though [some features are missing in templating library](https://github.com/sunng87/handlebars-rust#limited-but-essential-control-structure-built-in).

Example template file (default Rust model template):

```
use super::*;
{{#if is_object}}
use serde::{Serialize,Deserialize};
{{#if has_date}}
use chrono::Date;
{{/if}}
{{#if has_datetime}}
use chrono::DateTime;
{{/if}}

#[derive(Serialize,Deserialize,Default)]
pub struct {{name}} {
{{#each fields}}
    pub {{r name}}: {{type}},
{{/each}}
{{#if additional_fields}}
{{#each fields}}
    #[serde(skip)]
    pub {{r name}}: {{type}},
{{/each}}
{{/if}}
}
{{/if}}
{{#if is_array}}
pub type {{name}} = Vec<{{items.name}}>;
{{/if}}
```

Template white-space formatting can be a bit fiddly, so usage of a language formatter in Makefile or something similar is recommended.

## Helpers

Includes some built-in [custom helpers](https://handlebars-draft.knappi.org/guide/#custom-helpers), which can be used in templates:
```
* lowercase - lowercase
* uppercase - UPPERCASE
* snakecase - snake_case
* pascalcase - PascalCase
* screamingcase - SCREAMING_SNAKE_CASE
* camelcase - camelCase
* kebabcase - kebab-case
* r - Formats reserved keywords according to language spec (Rust example: type -> r#type). Kept short for convenience.
```

## Ignoring files

Ignoring files can be done with `.ramignore`, which follows `.gitignore` format:

```
src/some/file/to/ignore.rs
src/some/files/to/ignore/*.rs
src/some/**/*.rs
```

## TODO:
* Implement more formatting helpers
* Implement API generation from openapi paths-section
* Add more lang-specs for most used languages
* Test suite
* Document template context objects
