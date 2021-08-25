use mdbook::{
    book::{Book, BookItem},
    preprocess::{Preprocessor, PreprocessorContext},
};

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
            Error::NoClosingShortcode => "An opening shortcode had no matching closing shortcode",
            Error::UnterminatedString => "A string did not contain a closing quote",
        };
        write!(f, "{}", result)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

trait Shortcode {
    const NAME: &'static str;

    fn process_match(input: &str, attrs: Vec<&str>) -> String;

    // TODO custom error type
    fn process_raw(input: &str) -> Result<String> {
        // The start can contain attributes e.g. `{{< hint info >}}` or
        // `{{< details "Title" open}}`.
        let start_sequence = format!("{{< {}", Self::NAME);
        let end_sequence = format!("{{{{< /{} >}}}}", Self::NAME);

        let mut result = input.to_owned();

        for (i, _) in input.match_indices(&start_sequence) {
            // The index of the attributes start. For example:
            // {{< details "Title" open >}}
            //            ^ here
            let attrs_start_index = i + start_sequence.len();
            // The index of the end of the inner content. For example:
            // {{< details "Title" open >}}
            //                     here ^
            let attrs_end_index = match input[attrs_start_index..].find(">}}") {
                Some(i) => i,
                // TODO technically this is a different issue than the one below, so it shouldn't
                // use this enum variant.
                None => return Err(Error::NoClosingShortcode),
            };
            let attrs = split_attributes(&input[attrs_start_index..attrs_end_index])?;

            let content_start_index = attrs_end_index + 3;
            let content_end_index = match input[content_start_index..].find(&end_sequence) {
                Some(i) => i,
                // No closing tag.
                None => return Err(Error::NoClosingShortcode),
            };

            let content_range = content_start_index..content_end_index;

            result.replace_range(
                content_range.clone(),
                &Self::process_match(&input[content_range], attrs),
            );
        }

        Ok(result)
    }
}

fn split_attributes(raw_attrs: &str) -> Result<Vec<&str>> {
    let mut result = Vec::new();
    let mut attr_start_index = 0;
    let mut attr_end_index = 0;
    let mut in_quote = false;

    let raw_attrs = raw_attrs.trim();
    for (i, c) in raw_attrs.char_indices() {
        // TODO add more quote types. There is probably a better way of doing this.
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

    fn process_match(input: &str, _attributes: Vec<&str>) -> String {
        // They will be approximately the same length.
        let _result = String::with_capacity(input.len());
        let _columns = input.split("<--->");

        todo!();
    }
}

struct Hint;

impl Shortcode for Hint {
    const NAME: &'static str = "hint";

    fn process_match(_input: &str, _attrs: Vec<&str>) -> String {
        todo!();
    }
}

struct Tabs;

impl Shortcode for Tabs {
    const NAME: &'static str = "tabs";

    fn process_match(_input: &str, _attrs: Vec<&str>) -> String {
        todo!();
    }
}

struct Details;

impl Shortcode for Details {
    const NAME: &'static str = "details";

    fn process_match(_input: &str, _attrs: Vec<&str>) -> String {
        todo!();
    }
}

fn process_chapter(content: &str) -> Result<String> {
    let mut result = content.to_owned();

    result = Columns::process_raw(&result)?;
    result = Hint::process_raw(&result)?;
    result = Tabs::process_raw(&result)?;
    result = Details::process_raw(&result)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(split_attributes(&i), expected);
            }
        }
    }
}
