# Rusty API Modeler

Generates API models from openapi spec using language-agnostic templating and spec system. When ready, should support any or most of the programming languages.

Examples available in the following use cases:
* Rust (Rocket, Actix)
* Go (Echo)
* Postgresql schema
* C (simple structs)

However, a language can be implemented by supplying a language yaml file and some needed templates for generation. Contributions are welcome!

## Usage

Start off with an example `config.yaml`:

```
# Target lang spec
# Can be file path or built-in yaml spec
lang: "rust"

# Template files, optional and used for overriding built-in templates
templates:
  # Path for custom model template file
  model: "templates/custom_model.rs.template"

# File paths, optional
# This example places models to <output>/src/models
paths:
  root: "src",
  model: "models"

# Custom formatting rule example
# Can be used in templates with {{anglebrackets "something"}}
format:
  anglebrackets: "<{{value}}>"
```

Then simply run `ram -c config.yaml -i <path/to/openapi.yaml> -o <output/folder>` to run code generation.

## Templating

Supports using built-in or custom templates by configuration.

Templating uses [handlebars](https://handlebars-draft.knappi.org/guide) syntax, though [some features are missing in templating library](https://github.com/sunng87/handlebars-rust#limited-but-essential-control-structure-built-in).

Example template file (from default golang model template):
```
package model

{{#if has_datetime}}
import (
  "time"
)
{{/if}}

{{#if is_object}}
type {{pascalcase name}} struct {
{{#each properties}}
  {{ pascalcase name }} {{ type }} `json:"{{ camelcase name }}" {{ ext "x-go-custom-tag" }}`
{{/each}}
{{#if additional_properties}}
{{#with additional_properties}}
{{#each properties}}
  {{ pascalcase name }} {{type}} `json:"-" {{ ext "x-go-custom-tag" }}`
{{/each}}
{{/with}}
{{/if}}
}
{{/if}}
```

Template white-space formatting is cumbersome, so usage of a language formatter is recommended.

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
* ext - Returns extension value (eg. With definition `x-go-custom-tag: json:"-"`, `{{ext "x-go-custom-tag"}}` => `json:"-"`)
```

Also includes [all built-in helpers from handlebars lib](https://docs.rs/handlebars/3.0.0-beta.1/handlebars/#built-in-helpers).

## Ignoring files

Ignoring files can be done with `.ramignore`, which follows `.gitignore` format:
```
src/some/file/to/ignore.rs
src/some/files/to/ignore/*.rs
src/some/**/*.rs
```

Note that ignorefile currently only matches entries relative to current working directory, 
so for example ignorefile in different output directory won't get matched.

## TODO:
* Implement more formatting helpers
* Add more lang-specs for most used languages
* Test suite
* Document template context objects
