use handlebars::{handlebars_helper, Handlebars};
use inflector::Inflector;

handlebars_helper!(lowercase: |s: String| s.to_lowercase());
handlebars_helper!(uppercase: |s: String| s.to_uppercase());
handlebars_helper!(pascalcase: |s: String| s.to_pascal_case());
handlebars_helper!(snakecase: |s: String| s.to_snake_case());
handlebars_helper!(screamingcase: |s: String| s.to_screaming_snake_case());

fn register_common_helpers(hb: &mut Handlebars) {
    hb.register_helper("lowercase", Box::new(lowercase));
    hb.register_helper("uppercase", Box::new(uppercase));
    hb.register_helper("pascalcase", Box::new(pascalcase));
    hb.register_helper("snakecase", Box::new(snakecase));
    hb.register_helper("screamingcase", Box::new(screamingcase));
}
