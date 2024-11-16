use std::vec::IntoIter;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub struct PulldownHighlighter {
    syntaxset: SyntaxSet,
    themeset: ThemeSet,
}

/// A highlighter that can be instantiated once and used many times for better performance.
impl PulldownHighlighter {
    pub fn new() -> PulldownHighlighter {
        let syntaxset = SyntaxSet::load_defaults_newlines();
        let themeset = ThemeSet::load_defaults();

        PulldownHighlighter {
            syntaxset,
            themeset,
        }
    }

    /// Apply syntax highlighting to pulldown-cmark events using github theme.
    ///
    /// Take an iterator over pulldown-cmark's events, and (on success) return a new iterator
    /// where code blocks have been turned into HTML text blocks with syntax highlighting.
    ///
    /// Highly based on <https://gitlab.com/eguiraud/highlight-pulldown>.
    pub fn highlight<'a, It>(&self, events: It) -> Vec<Event<'a>>
    where
        It: Iterator<Item = Event<'a>>,
    {
        let mut in_code_block = false;

        let mut syntax = self.syntaxset.find_syntax_plain_text();

        let theme = self
            .themeset
            .themes
            .get("base16-ocean.dark")
            .expect("Couldn't find theme");

        let mut to_highlight = String::new();
        let mut out_events = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(lang) => {
                            syntax = self.syntaxset.find_syntax_by_token(&lang).unwrap_or(syntax)
                        }
                        CodeBlockKind::Indented => {}
                    }
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !in_code_block {
                        panic!("this should never happen");
                    }
                    let html =
                        highlighted_html_for_string(&to_highlight, &self.syntaxset, syntax, theme)
                            .expect("Couldn't highlight");

                    to_highlight.clear();
                    in_code_block = false;
                    out_events.push(Event::Html(CowStr::from(html)));
                }
                Event::Text(t) => {
                    if in_code_block {
                        to_highlight.push_str(&t);
                    } else {
                        out_events.push(Event::Text(t));
                    }
                }
                e => {
                    out_events.push(e);
                }
            }
        }

        out_events
    }
}

/// Apply syntax highlighting to pulldown-cmark.
///
/// Take an iterator over pulldown-cmark's events, and (on success) return a new iterator
/// where code blocks have been turned into HTML text blocks with syntax highlighting.
pub fn highlight<'a, It>(events: It) -> IntoIter<Event<'a>>
where
    It: Iterator<Item = Event<'a>>,
{
    let highlighter = PulldownHighlighter::new();
    highlighter.highlight(events).into_iter()
}
