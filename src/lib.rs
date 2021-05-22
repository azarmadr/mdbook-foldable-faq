use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::{Error, Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{Event, Options, Parser};
use pulldown_cmark_to_cmark::{cmark_with_options, Options as COptions};

pub struct FoldAQ;

impl Preprocessor for FoldAQ {
    fn name(&self) -> &str {
        "foldaq"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(FoldAQ::fold_txt(chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn escape_html(s: &str) -> String {
    let mut output = String::new();
    for c in s.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            _ => output.push(c),
        }
    }
    output
}

fn fold_txt(content: &str) -> Result<String> {
    let mut buf = String::with_capacity(content.len());
    let mut foldable_faq = String::new();
    let mut in_faq = false;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let events = Parser::new_ext(content, opts).map(|e| {
        /*
        if let Event::Start(Tag::Paragraph) = e.clone() {
            if 
        }
        */
        match e {
            Event::Text(t) => Event::Text(t
                                          .replace("#f", "<details><summary>")
                                          .replace("#q", "</summary>")
                                          .replace("#a", "</details>")
                                          .into()),
            _ => e,
        }
    });
    let events = events.filter_map(|e| Some(e));
    let mut opts = COptions::default();
    opts.newlines_after_codeblock = 1;
    cmark_with_options(events, &mut buf,  None, opts)
        .map(|_| buf)
        .map_err(|err| Error::msg(format!("Markdown Serialization failed: {}", err)))
}

impl FoldAQ {
    fn fold_txt(chapter: &mut Chapter) -> Result<String> {
        fold_txt(&chapter.content)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::fold_txt;

    #[test]
    fn check_folds() {
        let content = r#"# Chapter
## Sub

#f Do you know who I am? #q
hey,> < its me `choli ke peeche` #a

Text
"#;
        println!("{}",fold_txt(content).unwrap());
    }

    #[test]
    fn leaves_tables_untouched() {
        // Regression test.
        // Previously we forgot to enable the same markdwon extensions as mdbook itself.

        let content = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        // Markdown roundtripping removes some insignificant whitespace
        let expected = r#"# Heading

|Head 1|Head 2|
|------|------|
|Row 1|Row 2|"#;

        println!("{}", fold_txt(content).unwrap());
        println!("{}", expected);
        assert_eq!(expected, fold_txt(content).unwrap());
    }

    #[test]
    fn leaves_html_untouched() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML

        let content = r#"# Heading

<del>

*foo*

</del>
"#;

        // Markdown roundtripping removes some insignificant whitespace
        let expected = r#"# Heading

<del>

*foo*

</del>
"#;

        assert_eq!(expected, fold_txt(content).unwrap());
    }

    #[test]
    fn html_in_list() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML

        let content = r#"# Heading

1. paragraph 1
   ```
   code 1
   ```
2. paragraph 2
"#;

        // Markdown roundtripping removes some insignificant whitespace
        let expected = r#"# Heading

1. paragraph 1
   ````
   code 1
   ````
1. paragraph 2"#;

        assert_eq!(expected, fold_txt(content).unwrap());
    }
}
