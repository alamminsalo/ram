use handlebars::*;
use inflector::Inflector;

handlebars_helper!(lowercase: |s: str| s.to_lowercase());
handlebars_helper!(uppercase: |s: str| s.to_uppercase());
handlebars_helper!(pascalcase: |s: str| s.to_pascal_case());
handlebars_helper!(snakecase: |s: str| s.to_snake_case());
handlebars_helper!(screamingcase: |s: str| s.to_screaming_snake_case());
