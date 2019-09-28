# Rusty Api Modeler

Generates API models from openapi spec using language-agnostic templating and spec system. When ready, should support any or most of the programming languages.

# Usage

Start off with a configuration file like this:

```
# Example ram config file

# OpenAPI specification file path
openapi: "./res/petstore.yaml"

# Target lang spec
# Can be file path or built-in yaml spec
lang: "rust"

# Template files, optional and used for overriding built-in templates
templates:
  # Path for model template file
  model: "templates/model.rs.mustache"
  # API template path
  api: "templates/api.rs.mustache"

# File paths, optional
# This example places models to src/models
paths:
  root: "src",
  model: "models"
```

