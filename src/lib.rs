use mdbook::{
    book::{Book, BookItem},
    preprocess::{Preprocessor, PreprocessorContext},
};

// The CSS class names used are purposefully verbose to ensure they don't conflict with anything.

const START_OPENING_DELIMETER: &str = "{{#";
const START_CLOSING_DELIMETER: &str = "}}";
const END_OPENING_DELIMETER: &str = "{{/";
const END_CLOSING_DELIMETER: &str = "}}";

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ShortcodesProcessor;

impl Preprocessor for ShortcodesProcessor {
    fn name(&self) -> &str {
        "shortcodes"
    }

    fn run(
        &self,
        _ctx: &PreprocessorContext,
        mut book: Book,
    ) -> std::result::Result<Book, mdbook::errors::Error> {
        for item in &mut book.sections {
            if let BookItem::Chapter(chapter) = item {
                chapter.content = process_chapter(&chapter.content)?;
            }
        }
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

impl ShortcodesProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    NoClosingShortcode,
    UnterminatedString,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Error::NoClosingShortcode => "an opening shortcode had no matching closing shortcode",
            Error::UnterminatedString => "a string did not contain a closing quote",
        };
        write!(f, "{}", result)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

trait Shortcode {
    /// The name that is used to call the shortcode.
    const NAME: &'static str;
    /// Any code that should be placed once at the start of the page (e.g. css).
    const HEADER: &'static str;

    fn process_match(input: &str, attrs: Vec<&str>) -> String;

    // TODO custom error type
    fn process_raw(input: &str) -> Result<String> {
        // The start can contain attributes e.g. `{{#hint info}}` or `{{#details "Title" open}}`
        // so we only look for the opening delimiter followed by the name. The closing delimeter
        // (i.e. "}}") is taken into account later.
        let start_sequence = format!("{}{}", START_OPENING_DELIMETER, Self::NAME);
        let end_sequence = format!(
            "{}{}{}",
            END_OPENING_DELIMETER,
            Self::NAME,
            END_CLOSING_DELIMETER
        );

        let mut result = input.to_owned();

        for (i, _) in input.match_indices(&start_sequence) {
            // The index of the attributes start.
            // {{#columns 3em}}
            //           ^ here
            let attrs_start_index = i + start_sequence.len();
            // The index of the end of the attributes.
            // {{#columns 3em}}
            //               ^ here
            let attrs_end_index = match input[attrs_start_index..].find(START_CLOSING_DELIMETER) {
                Some(i) => attrs_start_index + i,
                // TODO technically this is a different error than the one below, so it shouldn't
                // use this error variant.
                None => return Err(Error::NoClosingShortcode),
            };
            let attrs = split_attrs(&input[attrs_start_index..attrs_end_index])?;

            // The index of the start of the content.
            // {{#columns 3em}}
            //                 ^ here (it is usually on a new line)
            let content_start_index = attrs_end_index + START_CLOSING_DELIMETER.len();
            // The index of the end of the content.
            // {{/columns}}
            // ^ here (note this is a closing tag)
            let content_end_index = match input[content_start_index..].find(&end_sequence) {
                Some(i) => content_start_index + i,
                // No closing tag.
                None => return Err(Error::NoClosingShortcode),
            };

            let content_range = content_start_index..content_end_index;

            result.replace_range(
                i..content_end_index + end_sequence.len(),
                &Self::process_match(&input[content_range], attrs),
            );
        }

        Ok(Self::HEADER.to_owned() + &result)
    }
}

fn split_attrs(raw_attrs: &str) -> Result<Vec<&str>> {
    let mut result = Vec::new();
    let mut attr_start_index = 0;
    let mut attr_end_index = 0;
    let mut in_quote = false;

    let raw_attrs = raw_attrs.trim();

    // TODO
    if raw_attrs.is_empty() {
        return Ok(Vec::new());
    }

    for (i, c) in raw_attrs.char_indices() {
        if is_quote(&c) {
            if in_quote {
                result.push(&raw_attrs[attr_start_index..i]);
            }
            attr_start_index = i + 1;
            in_quote = !in_quote;
        } else if c.is_whitespace() && !in_quote {
            if i != attr_start_index {
                result.push(&raw_attrs[attr_start_index..i]);
            }
            attr_start_index = i + 1;
        }
        attr_end_index = i;
    }

    if in_quote {
        return Err(Error::UnterminatedString);
    } else if attr_start_index <= attr_end_index {
        // `attr_start_index` is only greater than `attr_end_index` at the end of the loop
        // if the last char of the string was a quote that closed a string. Hence, this
        // block is only entered if the last character WASN'T a closing quote. Since,
        // whitespace has been stripped, we are guaranteed to have missed the last attribute
        // in the loop.
        result.push(&raw_attrs[attr_start_index..=attr_end_index])
    }

    Ok(result)
}

fn is_quote(c: &char) -> bool {
    *c == '\'' || *c == '"'
}

struct Columns;

impl Shortcode for Columns {
    const NAME: &'static str = "columns";
    const HEADER: &'static str = "
<style>
    .mdbook-shortcodes-columns-container {
        display: flex;
        margin: 0 -1em;
    }
    .mdbook-shortcodes-column {
        flex: 50%;
        padding: 0 1em;
    }
</style>
";

    fn process_match(input: &str, attrs: Vec<&str>) -> String {
        let padding = match attrs.len() {
            0 => None,
            1 => Some(attrs[0]),
            _ => panic!("too many arguments given to columns shortcode"),
        };
        let (container_style, column_style) = match padding {
            Some(p) => (
                format!("style=\"margin: 0 -{}\"", p),
                format!("style=\"padding: 0 {}\"", p),
            ),
            None => (String::new(), String::new()),
        };

        // Input and output will approximately be the same length.
        let mut result = String::with_capacity(input.len());
        result.push_str(&format!(
            "<div class=\"mdbook-shortcodes-columns-container\" {}>",
            container_style
        ));

        for column_content in input.split("{{#column}}") {
            result.push_str(&format!(
                "<div class=\"mdbook-shortcodes-column\" {}>",
                column_style
            ));
            result.push_str(column_content);
            result.push_str("</div>");
        }

        result.push_str("</div>");

        result
    }
}

struct Hint;

impl Shortcode for Hint {
    const NAME: &'static str = "hint";
    const HEADER: &'static str = "
<style>
    .mdbook-shortcodes-hint {
        padding: .5rem 2rem .5rem 1.75rem;
        border-inline-start: .5rem solid #fff;
        border-radius: .5rem;
    }

    .mdbook-shortcodes-hint-info {
        border-color: #6bf;
        background-color: rgba(102,187,255,.1);
    }

    .mdbook-shortcodes-hint-ok {
        border-color: #5b6;
        background-color: rgba(85,187,102,.1);
    }

    .mdbook-shortcodes-hint-warning {
        border-color: #fd6;
        background-color: rgba(255,221,102,.1);
    }

    .mdbook-shortcodes-hint-danger {
        border-color: #f66;
        background-color: rgba(255,102,102,.1);
    }
</style>
";

    fn process_match(input: &str, attrs: Vec<&str>) -> String {
        let ty = match attrs.len() {
            1 => attrs[0],
            _ => panic!("too many arguments given to columns shortcode"),
        };

        if let "info" | "ok" | "warning" | "danger" = ty {
            let mut result = String::new();
            result += &format!(
                "<div class=\"mdbook-shortcodes-hint mdbook-shortcodes-hint-{}\">",
                ty
            );
            result += input;
            result += "</div>";
            eprintln!("result: {}", result);
            result
        } else {
            panic!("unknown hint type");
        }
    }
}

struct Tabs;

impl Shortcode for Tabs {
    const NAME: &'static str = "tabs";
    const HEADER: &'static str = "";

    fn process_match(_input: &str, _attrs: Vec<&str>) -> String {
        todo!();
    }
}

fn process_chapter(content: &str) -> Result<String> {
    let mut result = content.to_owned();

    result = Columns::process_raw(&result)?;
    result = Hint::process_raw(&result)?;
    result = Tabs::process_raw(&result)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columns() {
        //
        let input = "
# Example
{{#columns}}

Column 1

{{#column}}

Column 2

{{/columns}}
";
        let expected = "
<style>
    .mdbook-shortcodes-columns-container {
        display: flex;
        margin: 0 -1em;
    }
    .mdbook-shortcodes-column {
        flex: 50%;
        padding: 0 1em;
    }
</style>

# Example
<div class=\"mdbook-shortcodes-columns-container\" ><div class=\"mdbook-shortcodes-column\" >

Column 1

</div><div class=\"mdbook-shortcodes-column\" >

Column 2

</div></div>
";
        assert_eq!(Columns::process_raw(input), Ok(expected.to_owned()));
    }

    #[test]
    fn test_split_attributes() {
        fn whitespace_variants(base: &str) -> Vec<String> {
            let mut result = vec![base.to_owned()];

            for w in [" ", "  "] {
                let mut temp = w.to_owned();
                temp.push_str(base);
                result.push(temp);

                let mut temp = base.to_owned();
                temp.push_str(w);
                result.push(temp);
            }

            result
        }

        let cases: Vec<(&str, Result<Vec<&str>>)> = vec![
            ("", Ok(Vec::new())),
            ("my name is john", Ok(vec!["my", "name", "is", "john"])),
            ("c", Ok(vec!["c"])),
            ("c a", Ok(vec!["c", "a"])),
            ("\"d\" \"q\"", Ok(vec!["d", "q"])),
            ("\"s\" \"q\"", Ok(vec!["s", "q"])),
            (
                "\"Multiple words in quotes\" foo 'bar'",
                Ok(vec!["Multiple words in quotes", "foo", "bar"]),
            ),
            ("\"Unterminated string", Err(Error::UnterminatedString)),
            ("Unterminated string\"", Err(Error::UnterminatedString)),
        ];

        for (input, expected) in cases {
            for i in whitespace_variants(input) {
                assert_eq!(split_attrs(&i), expected);
            }
        }
    }
}
