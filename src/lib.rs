use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct ShortcodesProcessor;

impl Preprocessor for ShortcodesProcessor {
    fn name(&self) -> &str {
        "shortcodes"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let _ = &mut book;
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                // TODO remove unwrap
                chapter.content = process_chapter(&chapter.content).unwrap();
            }
        });
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

trait Shortcode {
    const START_SEQUENCE: &'static str;
    const END_SEQUENCE: &'static str;

    fn process_match(input: &str) -> String;

    // TODO custom error type
    fn process_raw(input: &str) -> Result<String, ()> {
        let mut result = input.to_owned();

        for (i, _) in input.match_indices(Self::START_SEQUENCE) {
            let start_index = i + Self::START_SEQUENCE.len();
            let end_index = match input[start_index..].find(Self::END_SEQUENCE) {
                Some(i) => i,
                // No closing tag.
                None => return Err(()),
            };

            let match_range = start_index..end_index;

            result.replace_range(
                match_range.clone(),
                &Self::process_match(&input[match_range]),
            );
        }

        Ok(result)
    }
}

struct Columns;

impl Shortcode for Columns {
    const START_SEQUENCE: &'static str = "{{< columns >}}";
    const END_SEQUENCE: &'static str = "{{< /columns >}}";

    fn process_match(_input: &str) -> String {
        todo!();
    }
}

fn process_chapter(content: &str) -> Result<String, ()> {
    let mut result = content.to_owned();

    result = Columns::process_raw(&result)?;

    Ok(result)
}
