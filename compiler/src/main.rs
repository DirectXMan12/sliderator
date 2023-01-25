// Copyright 2021 Solly Ross

use pandoc_ast::{Block, Inline, MutVisitor, MetaValue, Attr};
use std::io::{Write as _, self};
use syntect::{util::LinesWithEndings, html::ClassedHTMLGenerator, parsing::SyntaxSet};

mod escape;

fn has_class(attrs: &Attr, target_class: &str) -> bool {
    attrs.1.iter().find(|class| *class == target_class).is_some()
}

fn key_value<'a, 'k>(attrs: &'a Attr, target_key: &'k str) -> Option<&'a str> {
    attrs.2.iter().find(|(key, _)| key == target_key).map(|(_, value)| &**value)
}

/// checks if this "wrapper" (div, span) is supposed to be a different tag
/// It's supposed to be a different tag if:
/// - it has the tag key, whose value will be returned and removed, or
/// - a single class name that matches some fixed set (which will be removed and returned):
///   * figure
///   * figcaption
fn get_and_remove_tag<'a>(attrs: &'a mut Attr) -> Option<String> {
    let tag = if let Some(type_attr) = attrs.2.iter().position(|(key, _)| key == "tag").map(|ind| attrs.2.remove(ind)).map(|(_, val)| val) {
        Some(type_attr)
    } else if attrs.1.len() == 1 {
        match &*attrs.1[0] {
            "figure" | "details" | "summary" | "figcaption" => {
                Some(attrs.1.remove(0))
            },
            _ => None
        }
    } else {
        None
    };

    match &tag {
        Some(typ) if typ == "figure" => {
            if key_value(attrs, "slot").is_none() {
                attrs.2.push(("slot".into(), "figure".into()));
            }
        },
        _ => {},
    }

    tag
}

// TODO: lock stdout?

struct SlidesVisitor<'s, W: io::Write> {
    /// the next available footnote
    footnote_index: usize,

    /// are we currently in a slide element (no at the start, yes afterwards)
    in_slide: bool,

    /// the slide index
    slide_ind: usize,

    out: W,

    syntax_set: &'s SyntaxSet,

    footnote_buffer: Vec<u8>,
}
impl <'s, W: io::Write> SlidesVisitor<'s, W> {
    fn inlines_with_tag<'a>(&mut self, tag: &'a str, contents: &mut Vec<Inline>) {
        write!(self.out, "<{}>", tag);
        self.visit_vec_inline(contents);
        write!(self.out, "</{}>", tag);
    }

    // TODO: make these write directly
    fn html_escape<'a>(&self, s: &'a str) -> String {
        let mut out = String::with_capacity(s.len());
        escape::escape_html(&mut out, s);
        out
    }
    fn href_escape<'a>(&self, s: &'a str) -> String {
        let mut out = String::with_capacity(s.len());
        escape::escape_href(&mut out, s);
        out
    }

    fn end_slide(&mut self) {
        write!(self.out, "<ol slot=\"footnotes\">");
        self.out.write_all(&self.footnote_buffer);
        self.footnote_buffer.clear();
        write!(self.out, "</ol></pres-slide>");
    }
}
impl <'s, W: io::Write> MutVisitor for SlidesVisitor<'s, W> {
    // TODO: newlines (search for nl)
    fn visit_block(&mut self, block: &mut Block) {
        use Block::*;
        match block {
            Plain(_) => {
                self.walk_block(block);
            }
            Para(_) => {
                write!(self.out, "<p>");
                self.walk_block(block);
                write!(self.out, "</p>");
            }
            LineBlock(_) => {
                // TODO: pandoc wraps this in a line-block div sometimes
                // otherwise calling linesToPara on it,
                // so this might be wrong
                self.walk_block(block);
            }
            CodeBlock(attrs, txt) => {
                write!(self.out, "<pre><code");
                self.visit_attr(attrs);

                if let Some(syn) = attrs.1.iter().next().and_then(|cl| self.syntax_set.find_syntax_by_token(cl)) {
                    let mut gen = ClassedHTMLGenerator::new(&syn, &self.syntax_set);
                    for line in LinesWithEndings::from(txt) {
                        gen.parse_html_for_line(&line);
                    }
                    write!(self.out, ">{}</code></pre>", gen.finalize());
                } else {
                    write!(self.out, ">{}</code></pre>", self.html_escape(txt));
                }
            }
            RawBlock(format, txt) => {
                match format.0.as_ref() /* TODO(directxman12): use or patterns when stable */ {
                    "html" | "html5" | "html4" => {
                        if txt == "<figure>" {
                            write!(self.out, "<figure slot=\"figure\">");
                        } else {
                            write!(self.out, "{}", txt /* use as-is */);
                        }
                    }
                    "latex" | "tex" => {
                        // TODO: ??
                    }
                    _ => {
                        // TODO: report this
                    }
                }
            }
            BlockQuote(_) => {
                write!(self.out, "<bq>");
                self.walk_block(block);
                write!(self.out, "</bq>");
            }
            OrderedList((start_ind, _num_style, _num_delim), items) => {
                // TODO: style using type attr
                if *start_ind != 1 {
                    write!(self.out, "<ol start=\"{}\">", start_ind);
                } else {
                    write!(self.out, "<ol>");
                }
                for item in items {
                    write!(self.out, "<li>");
                    self.visit_vec_block(item);
                    write!(self.out, "</li>");
                }
                write!(self.out, "</ol>");
            }
            BulletList(items) => {
                write!(self.out, "<ul>");
                for item in items {
                    write!(self.out, "<li>");
                    self.visit_vec_block(item);
                    write!(self.out, "</li>");
                }
                write!(self.out, "</ul>");
            }
            DefinitionList(items) => {
                write!(self.out, "<dl>");
                for (term, defs) in items {
                    write!(self.out, "<dt>");
                    self.visit_vec_inline(term);
                    write!(self.out, "</dt>");

                    for def in defs {
                        write!(self.out, "<dd>");
                        self.visit_vec_block(def);
                        write!(self.out, "</dd>");
                    }
                }
                write!(self.out, "</dl>");
            }
            Header(lvl, attrs, contents) => {
                if *lvl == 1 {
                    if self.in_slide {
                        self.end_slide();
                    }
                    self.in_slide = true;
                    if let Some(master_name) = key_value(attrs, "master") {
                        write!(self.out, "<pres-slide master=\"{master}\" id=\"slide-{ind}\">", master=master_name, ind=self.slide_ind);
                    } else {
                        write!(self.out, "<pres-slide id=\"slide-{}\">", self.slide_ind);
                    }
                    self.slide_ind+=1;
                }
                write!(self.out, "<h{}", lvl);
                self.visit_attr(attrs);
                write!(self.out, ">");
                self.visit_vec_inline(contents);
                write!(self.out, "</h{}>", lvl);
            }
            HorizontalRule => {
                write!(self.out, "<hr/>");
            }
            Table(caption, col_aligns, col_widths, headers, cells) => {
                // TODO: all this jazz
            }
            Div(attrs, contents) => {
                // TODO(directxman12): pandoc does special handling here for things with the
                // section class
                // TODO(directxman12): pandoc handles some other stuff here specially (roles,
                // notes, etc)

                let tag = get_and_remove_tag(attrs).unwrap_or("div".into());

                write!(self.out, "<{}", tag);
                self.visit_attr(attrs);
                write!(self.out, ">");
                self.visit_vec_block(contents);
                write!(self.out, "</{}>", tag);
            }
            Null => {}
        }
    }
    fn visit_attr(&mut self, attr: &mut Attr) {
        let (id, classes, kv_pairs) = attr;

        // TODO: quote
        if id != "" {
            write!(self.out, " id=\"{}\"", id);
        }

        if classes.len() > 0 {
            write!(self.out, " class=\"{}\"", classes.join(" "));
        }

        for (k, v) in kv_pairs {
            write!(self.out, " {}=\"{}\"", k, v);
        }

        self.walk_attr(attr)
    }
    fn visit_inline(&mut self, inline: &mut Inline) {
        use Inline::*;
        match inline {
            Str(txt) => {
                write!(self.out, "{}", self.html_escape(txt));
            }
            Emph(contents) => {
                self.inlines_with_tag("em", contents);
            }
            Strong(contents) => {
                self.inlines_with_tag("strong", contents);
            }
            Strikeout(contents) => {
                // TODO: del or s -- pandoc uses del
                self.inlines_with_tag("del", contents);
            }
            Superscript(contents) => {
                self.inlines_with_tag("sup", contents);
            }
            Subscript(contents) => {
                self.inlines_with_tag("sub", contents);
            }
            SmallCaps(contents) => {
                write!(self.out, "<span class=\"smallcaps\">");
                self.visit_vec_inline(contents);
                write!(self.out, "</span>");
            }
            Quoted(_quote_type, contents) => {
                // TODO(directxman12): quote type (generally can ignore since using tag)
                self.inlines_with_tag("q", contents);
            }
            Cite(citations, contents) => {
                // TODO: unwords $ map citationId citations
                // span class=citation data-cites=toValue citiationIds
            }
            Code(attrs, txt) => {
                if !has_class(attrs, "inline") {
                    attrs.1.push("inline".into());
                }
                write!(self.out, "<code");
                self.visit_attr(attrs);
                write!(self.out, ">{}</code>", self.html_escape(txt));
            }
            Space => { write!(self.out, " "); }
            SoftBreak => {
                // TODO: depends on Wrap (WrapNone, WrapAuto --> " ", WrapPreserve --> "\n")
                write!(self.out, "\n");
            }
            LineBreak => { write!(self.out, "<br/>"); }
            Math(math_type, txt) => {
                // TODO: ??
            }
            RawInline(format, txt) => {
                match format.0.as_ref() /* TODO(directxman12): use or patterns when stable */ {
                    "html" | "html5" | "html4" => {
                        write!(self.out, "{}", txt /* use as-is */);
                    }
                    "latex" | "tex" => {
                        // TODO: ??
                    }
                    _ => {
                        // TODO: report this
                    }
                }
            }
            Link(attrs, desc, target) => {
                // TODO: obfuscate mailto links??? (pandoc does this)
                let (url, title) = target;
                write!(self.out, "<a");
                self.visit_attr(attrs);
                if title != "" {
                    write!(self.out, " title=\"{title}\"", title=self.html_escape(title));
                }
                write!(self.out, " href=\"{}\">", self.href_escape(url));
                self.visit_vec_inline(desc);
                write!(self.out, "</a>");
            }
            Image(attrs, desc, target) => {
                // TODO(directxman12): handle different media from URI --> video, audio tags
                // TODO(directxman12): handle `fig:` title from pandoc source

                let (url, title) = target;

                let is_figure = title.starts_with("fig:");
                if is_figure {
                    *title = title[4..].into();
                    if key_value(attrs, "slot").is_none() {
                        attrs.2.push(("slot".into(), "figure".into()));
                    }
                    write!(self.out, "<figure");
                    self.visit_attr(attrs);
                    write!(self.out, "><img");
                } else {
                    write!(self.out, "<img");
                }
                self.visit_attr(attrs);
                if title != "" {
                    // moz-dev says generally not to use the title attribute on images, so just use
                    // alt
                    write!(self.out, " alt=\"{txt}\"", txt=self.html_escape(title));
                }
                write!(self.out, " src=\"{}\">", self.href_escape(url));
                write!(self.out, "</img>");
                if is_figure {
                    write!(self.out, "</figure>");
                }
            }
            Note(contents) => {
                // TODO: blockListToNote, a to #stuff with role="doc-noteref"
                let num = self.footnote_index;
                self.footnote_index += 1;
                write!(self.out, "<a id=\"fnref-{num}\" class=\"footnote-ref\" href=\"#fn-{num}\" role=\"doc-noteref\">{num}</a>", num=num);

                // NB(directxman12): can't use slots in a list element unfortunately :-/
                write!(self.footnote_buffer, "<li value=\"{num}\" role=\"doc-endnote\" id=\"fn-{num}\">", num=num);
                let mut sub_visitor = SlidesVisitor{
                    slide_ind: self.slide_ind,
                    out: &mut self.footnote_buffer,
                    in_slide: self.in_slide,
                    footnote_buffer: Vec::new(), // TODO: deal with this better
                    footnote_index: self.footnote_index, // TODO: restore this later
                    syntax_set: self.syntax_set,
                };
                if let Some(Block::Para(ref mut end)) = contents.last_mut() {
                    end.push(Inline::Space);
                    end.push(Inline::Link(("".into(), vec!["footnote-back".into()], vec![("role".into(), "doc-backlink".into())]), vec![Inline::Str("↩".into())], (format!("#fnref-{}", num), "".into())));
                }
                sub_visitor.visit_vec_block(contents);
                write!(self.footnote_buffer, "</li>");
            },
            Span(attrs, contents) => {
                let tag = get_and_remove_tag(attrs).unwrap_or("span".into());
                write!(self.out, "<{}", tag);
                self.visit_attr(attrs);
                write!(self.out, ">");
                self.visit_vec_inline(contents);
                write!(self.out, "</{}>", tag);
            }
        }
    }
    fn visit_meta(&mut self, _key: &str, meta: &mut MetaValue) {
        // TODO: ??
        self.walk_meta(meta)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Read};
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (prefix, suffix): (String, String) = {
        let mut template = String::new();
        let filename = std::env::args().skip(1).next().ok_or("must specify template name")?;
        std::fs::File::open(filename)?.read_to_string(&mut template)?;

        let mut parts = template.split("$body$");
        (parts.next().ok_or("can't get template prefix")?.into(), parts.next().ok_or("can't get template suffix")?.into())
    };

    print!("{}", prefix);

    let syntax_set: SyntaxSet = syntect::dumps::from_binary(include_bytes!("../../third_party/syntax-highlighting/default_newlines.packdump"));
    pandoc_ast::filter(input, |mut doc| {
        let out = io::stdout();
        let mut visitor = SlidesVisitor{
            syntax_set: &syntax_set,
            slide_ind: 0,
            footnote_index: 1,
            in_slide: false,
            out: out.lock(),
            footnote_buffer: Vec::new(),
        };
        visitor.walk_pandoc(&mut doc);
        if visitor.in_slide {
            // TODO: move into helper
            visitor.end_slide();
        }

        doc
    });

    print!("{}", suffix);

    Ok(())
}
