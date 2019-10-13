use handlebars::{handlebars_helper, Handlebars};
use inflector::Inflector;

handlebars_helper!(lowercase: |s: str| s.to_lowercase());
handlebars_helper!(uppercase: |s: str| s.to_uppercase());
handlebars_helper!(pascalcase: |s: str| s.to_pascal_case());
handlebars_helper!(snakecase: |s: str| s.to_snake_case());
handlebars_helper!(screamingcase: |s: str| s.to_screaming_snake_case());
handlebars_helper!(camelcase: |s: str| s.to_camel_case());
handlebars_helper!(kebabcase: |s: str| s.to_kebab_case());

pub fn register_helpers(hb: &mut Handlebars) {
    hb.register_helper("lowercase", Box::new(lowercase));
    hb.register_helper("uppercase", Box::new(uppercase));
    hb.register_helper("pascalcase", Box::new(pascalcase));
    hb.register_helper("snakecase", Box::new(snakecase));
    hb.register_helper("screamingcase", Box::new(screamingcase));
    hb.register_helper("camelcase", Box::new(camelcase));
    hb.register_helper("kebabcase", Box::new(kebabcase));
}
