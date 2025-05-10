//! This crate defines a
//! [Wadler-style](http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf)
//! pretty-printing API.
//!
//! Start with the static functions of [Doc](enum.Doc.html).
//!
//! ## Quick start
//!
//! Let's pretty-print simple sexps!  We want to pretty print sexps like
//!
//! ```lisp
//! (1 2 3)
//! ```
//! or, if the line would be too long, like
//!
//! ```lisp
//! ((1)
//!  (2 3)
//!  (4 5 6))
//! ```
//!
//! A _simple symbolic expression_ consists of a numeric _atom_ or a nested ordered _list_ of
//! symbolic expression children.
//!
//! ```rust
//! # use pretty::*;
//! enum SExp {
//!     Atom(u32),
//!     List(Vec<SExp>),
//! }
//! use SExp::*;
//! # fn main() { }
//! ```
//!
//! We define a simple conversion to a [Doc](enum.Doc.html).  Atoms are rendered as strings; lists
//! are recursively rendered, with spaces between children where appropriate.  Children are
//! [nested]() and [grouped](), allowing them to be laid out in a single line as appropriate.
//!
//! ```rust
//! # use pretty::*;
//! # enum SExp {
//! #     Atom(u32),
//! #     List(Vec<SExp>),
//! # }
//! # use SExp::*;
//! impl SExp {
//!     /// Return a pretty printed format of self.
//!     pub fn to_doc(&self) -> RcDoc<()> {
//!         match *self {
//!             Atom(ref x) => RcDoc::as_string(x),
//!             List(ref xs) =>
//!                 RcDoc::text("(")
//!                     .append(RcDoc::intersperse(xs.into_iter().map(|x| x.to_doc()), Doc::line()).nest(1).group())
//!                     .append(RcDoc::text(")"))
//!         }
//!     }
//! }
//! # fn main() { }
//! ```
//!
//! Next, we convert the [Doc](enum.Doc.html) to a plain old string.
//!
//! ```rust
//! # use pretty::*;
//! # enum SExp {
//! #     Atom(u32),
//! #     List(Vec<SExp>),
//! # }
//! # use SExp::*;
//! # impl SExp {
//! #     /// Return a pretty printed format of self.
//! #     pub fn to_doc(&self) -> BoxDoc<()> {
//! #         match *self {
//! #             Atom(ref x) => BoxDoc::as_string(x),
//! #             List(ref xs) =>
//! #                 BoxDoc::text("(")
//! #                     .append(BoxDoc::intersperse(xs.into_iter().map(|x| x.to_doc()), Doc::line()).nest(1).group())
//! #                     .append(BoxDoc::text(")"))
//! #         }
//! #     }
//! # }
//! impl SExp {
//!     pub fn to_pretty(&self, width: usize) -> String {
//!         let mut w = Vec::new();
//!         self.to_doc().render(width, &mut w).unwrap();
//!         String::from_utf8(w).unwrap()
//!     }
//! }
//! # fn main() { }
//! ```
//!
//! And finally we can test that the nesting and grouping behaves as we expected.
//!
//! ```rust
//! # use pretty::*;
//! # enum SExp {
//! #     Atom(u32),
//! #     List(Vec<SExp>),
//! # }
//! # use SExp::*;
//! # impl SExp {
//! #     /// Return a pretty printed format of self.
//! #     pub fn to_doc(&self) -> BoxDoc<()> {
//! #         match *self {
//! #             Atom(ref x) => BoxDoc::as_string(x),
//! #             List(ref xs) =>
//! #                 BoxDoc::text("(")
//! #                     .append(BoxDoc::intersperse(xs.into_iter().map(|x| x.to_doc()), Doc::line()).nest(1).group())
//! #                     .append(BoxDoc::text(")"))
//! #         }
//! #     }
//! # }
//! # impl SExp {
//! #     pub fn to_pretty(&self, width: usize) -> String {
//! #         let mut w = Vec::new();
//! #         self.to_doc().render(width, &mut w).unwrap();
//! #         String::from_utf8(w).unwrap()
//! #     }
//! # }
//! # fn main() {
//! let atom = SExp::Atom(5);
//! assert_eq!("5", atom.to_pretty(10));
//! let list = SExp::List(vec![SExp::Atom(1), SExp::Atom(2), SExp::Atom(3)]);
//! assert_eq!("(1 2 3)", list.to_pretty(10));
//! assert_eq!("\
//! (1
//!  2
//!  3)", list.to_pretty(5));
//! # }
//! ```
//!
//! ## Advanced usage
//!
//! There's a more efficient pattern that uses the [DocAllocator](trait.DocAllocator.html) trait, as
//! implemented by [BoxAllocator](struct.BoxAllocator.html), to allocate
//! [DocBuilder](struct.DocBuilder.html) instances.  See
//! [examples/trees.rs](https://github.com/freebroccolo/pretty.rs/blob/master/examples/trees.rs#L39)
//! for this approach.

mod alloc;
pub mod block;
mod builder;
mod doc;
mod render;

#[cfg(feature = "termcolor")]
pub use termcolor;

pub use alloc::{Arena, BoxAllocator, DocAllocator, RcAllocator};
pub use block::{Affixes, BlockDoc};
pub use builder::DocBuilder;
pub use doc::{BoxDoc, BuildDoc, Doc, DocPtr, RcDoc, RefDoc};
pub use render::{FmtWrite, IoWrite, Render, RenderAnnotated};

type SmallText = arrayvec::ArrayString<[u8; 22]>;

/// Trait for types which can be converted to a `Document`
pub trait Pretty<'a, D, A = ()>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    /// Converts `self` into a document
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A>;
}

impl<'a, A> Pretty<'a, BoxAllocator, A> for BoxDoc<'a, A>
where
    A: 'a,
{
    fn pretty(self, allocator: &'a BoxAllocator) -> DocBuilder<'a, BoxAllocator, A> {
        DocBuilder(allocator, self.into())
    }
}

impl<'a, A> Pretty<'a, RcAllocator, A> for RcDoc<'a, A>
where
    A: 'a,
{
    fn pretty(self, allocator: &'a RcAllocator) -> DocBuilder<'a, RcAllocator, A> {
        DocBuilder(allocator, self.into())
    }
}

impl<'a, A> Pretty<'a, Arena<'a, A>, A> for RefDoc<'a, A>
where
    A: 'a,
{
    fn pretty(self, allocator: &'a Arena<'a, A>) -> DocBuilder<'a, Arena<'a, A>, A> {
        DocBuilder(allocator, self.into())
    }
}

impl<'a, D, A> Pretty<'a, D, A> for BuildDoc<'a, D::Doc, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        DocBuilder(allocator, self)
    }
}

impl<'a, D, A> Pretty<'a, D, A> for Doc<'a, D::Doc, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        DocBuilder(allocator, self.into())
    }
}

impl<'a, D, A> Pretty<'a, D, A> for DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, _: &'a D) -> DocBuilder<'a, D, A> {
        self
    }
}

impl<'a, D, A, T> Pretty<'a, D, A> for Option<T>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
    T: Pretty<'a, D, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        match self {
            Some(x) => x.pretty(allocator),
            None => allocator.nil(),
        }
    }
}

impl<'a, D, A> Pretty<'a, D, A> for &'a str
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        allocator.text(self)
    }
}

impl<'a, D, A> Pretty<'a, D, A> for &'a String
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        self[..].pretty(allocator)
    }
}

impl<'a, D, A> Pretty<'a, D, A> for String
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        allocator.text(self)
    }
}

impl<'a, D, A, S> Pretty<'a, D, A> for std::borrow::Cow<'a, S>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
    S: ?Sized + ToOwned,
    &'a S: Pretty<'a, D, A>,
    S::Owned: Pretty<'a, D, A>,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {
        match self {
            std::borrow::Cow::Borrowed(s) => s.pretty(allocator),
            std::borrow::Cow::Owned(s) => s.pretty(allocator),
        }
    }
}

/// Concatenates a number of documents (or values that can be converted into a document via the
/// `Pretty` trait, like `&str`)
///
/// ```
/// use pretty::{docs, Arena, DocAllocator};
/// let arena = &Arena::<()>::new();
/// let doc = docs![
///     arena,
///     "let",
///     arena.softline(),
///     "x",
///     arena.softline(),
///     "=",
///     arena.softline(),
///     Some("123"),
/// ];
/// assert_eq!(doc.1.pretty(80).to_string(), "let x = 123");
/// ```
#[macro_export]
macro_rules! docs {
    ($alloc: expr, $first: expr $(,)?) => {
        $crate::Pretty::pretty($first, $alloc)
    };
    ($alloc: expr, $first: expr $(, $rest: expr)+ $(,)?) => {{
        let mut doc = $crate::Pretty::pretty($first, $alloc);
        $(
            doc = doc.append($rest);
        )*
        doc
    }}
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    macro_rules! chain {
        ($first: expr $(, $rest: expr)* $(,)?) => {{
            #[allow(unused_mut)]
            let mut doc = DocBuilder(&BoxAllocator, $first.into());
            $(
                doc = doc.append($rest);
            )*
            doc.into_doc()
        }}
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn doc_size() {
        // Safeguard against accidentally growing Doc
        assert_eq!(8 * 3, std::mem::size_of::<Doc<RefDoc>>());
    }

    macro_rules! test {
        ($size:expr, $actual:expr, $expected:expr) => {
            let mut s = String::new();
            $actual.render_fmt($size, &mut s).unwrap();
            difference::assert_diff!(&s, $expected, "\n", 0);
        };
        ($actual:expr, $expected:expr) => {
            test!(70, $actual, $expected)
        };
    }

    #[test]
    fn box_doc_inference() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("test")
                .append(BoxDoc::line())
                .append(BoxDoc::text("test")),
        );

        test!(doc, "test test");
    }

    #[test]
    fn newline_in_text() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("test").append(
                BoxDoc::line()
                    .append(BoxDoc::text("\"test\n     test\""))
                    .nest(4),
            ),
        );

        test!(5, doc, "test\n    \"test\n     test\"");
    }

    #[test]
    fn forced_newline() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("test")
                .append(BoxDoc::hardline())
                .append(BoxDoc::text("test")),
        );

        test!(doc, "test\ntest");
    }

    #[test]
    fn space_do_not_reset_pos() {
        let doc: BoxDoc<()> = BoxDoc::group(BoxDoc::text("test").append(BoxDoc::line()))
            .append(BoxDoc::text("test"))
            .append(BoxDoc::group(BoxDoc::line()).append(BoxDoc::text("test")));

        test!(9, doc, "test test\ntest");
    }

    // Tests that the `BoxDoc::hardline()` does not cause the rest of document to think that it fits on
    // a single line but instead breaks on the `BoxDoc::line()` to fit with 6 columns
    #[test]
    fn newline_does_not_cause_next_line_to_be_to_long() {
        let doc: RcDoc<()> = RcDoc::group(
            RcDoc::text("test").append(RcDoc::hardline()).append(
                RcDoc::text("test")
                    .append(RcDoc::line())
                    .append(RcDoc::text("test")),
            ),
        );

        test!(6, doc, "test\ntest\ntest");
    }

    #[test]
    fn newline_after_group_does_not_affect_it() {
        let arena = Arena::<()>::new();
        let doc = arena.text("x").append(arena.line()).append("y").group();

        test!(100, doc.append(arena.hardline()).1, "x y\n");
    }

    #[test]
    fn block() {
        let doc: RcDoc<()> = RcDoc::group(
            RcDoc::text("{")
                .append(
                    RcDoc::line()
                        .append(RcDoc::text("test"))
                        .append(RcDoc::line())
                        .append(RcDoc::text("test"))
                        .nest(2),
                )
                .append(RcDoc::line())
                .append(RcDoc::text("}")),
        );

        test!(5, doc, "{\n  test\n  test\n}");
    }

    #[test]
    fn block_with_hardline() {
        let doc: RcDoc<()> = RcDoc::group(
            RcDoc::text("{")
                .append(
                    RcDoc::line()
                        .append(RcDoc::text("test"))
                        .append(RcDoc::hardline())
                        .append(RcDoc::text("test"))
                        .nest(2),
                )
                .append(RcDoc::line())
                .append(RcDoc::text("}")),
        );

        test!(10, doc, "{\n  test\n  test\n}");
    }

    #[test]
    fn block_with_hardline_negative_nest() {
        let doc: RcDoc<()> = RcDoc::group(
            RcDoc::text("{")
                .append(
                    RcDoc::line()
                        .append(RcDoc::text("test"))
                        .append(RcDoc::hardline())
                        .append(RcDoc::text("test"))
                        .nest(-2),
                )
                .append(RcDoc::line())
                .append(RcDoc::text("}")),
        );

        test!(10, doc, "{\ntest\ntest\n}");
    }

    #[test]
    fn line_comment() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("{")
                .append(
                    BoxDoc::line()
                        .append(BoxDoc::text("test"))
                        .append(BoxDoc::line())
                        .append(BoxDoc::text("// a").append(BoxDoc::hardline()))
                        .append(BoxDoc::text("test"))
                        .nest(2),
                )
                .append(BoxDoc::line())
                .append(BoxDoc::text("}")),
        );

        test!(14, doc, "{\n  test\n  // a\n  test\n}");
    }

    #[test]
    fn annotation_no_panic() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("test")
                .annotate(())
                .append(BoxDoc::hardline())
                .annotate(())
                .append(BoxDoc::text("test")),
        );

        test!(doc, "test\ntest");
    }

    fn nest_on_line(doc: BoxDoc<'static, ()>) -> BoxDoc<'static, ()> {
        BoxDoc::softline().append(BoxDoc::nesting(move |n| {
            let doc = doc.clone();
            BoxDoc::column(move |c| {
                if n == c {
                    BoxDoc::text("  ").append(doc.clone()).nest(2)
                } else {
                    doc.clone()
                }
            })
        }))
    }

    #[test]
    fn hang_lambda1() {
        let doc = chain![
            chain!["let", BoxDoc::line(), "x", BoxDoc::line(), "="].group(),
            nest_on_line(chain![
                "\\y ->",
                chain![BoxDoc::line(), "y"].nest(2).group()
            ]),
        ]
        .group();

        test!(doc, "let x = \\y -> y");
        test!(
            8,
            doc,
            r"let x =
  \y ->
    y"
        );
        test!(
            14,
            doc,
            r"let x = \y ->
  y"
        );
    }

    #[test]
    fn hang_comment() {
        let body = chain!["y"].nest(2).group();
        let doc = chain![
            chain!["let", BoxDoc::line(), "x", BoxDoc::line(), "="].group(),
            nest_on_line(chain![
                "\\y ->",
                nest_on_line(chain!["// abc", BoxDoc::hardline(), body])
            ]),
        ]
        .group();

        test!(8, doc, "let x =\n  \\y ->\n    // abc\n    y");
        test!(14, doc, "let x = \\y ->\n  // abc\n  y");
    }

    #[test]
    fn union() {
        let doc = chain![
            chain!["let", BoxDoc::line(), "x", BoxDoc::line(), "="].group(),
            nest_on_line(chain![
                "(",
                chain![
                    BoxDoc::line_(),
                    chain!["x", ","].group(),
                    BoxDoc::line(),
                    chain!["1234567890", ","].group()
                ]
                .nest(2)
                .group(),
                BoxDoc::line_().append(")"),
            ])
        ]
        .group();

        test!(doc, "let x = (x, 1234567890,)");
        test!(8, doc, "let x =\n  (\n    x,\n    1234567890,\n  )");
        test!(14, doc, "let x = (\n  x,\n  1234567890,\n)");
    }

    fn hang2(
        from: BoxDoc<'static, ()>,
        body_whitespace: BoxDoc<'static, ()>,
        body: BoxDoc<'static, ()>,
        trailer: BoxDoc<'static, ()>,
    ) -> BoxDoc<'static, ()> {
        let body1 = body_whitespace
            .append(body.clone())
            .nest(2)
            .group()
            .append(trailer.clone());
        let body2 = BoxDoc::hardline()
            .append(body.clone())
            .nest(2)
            .group()
            .append(trailer.clone());

        let single = from.clone().append(body1.clone()).group();

        let hang = from.clone().append(body2).group();

        let break_all = from.append(body1).group().nest(2);

        BoxDoc::group(single.union(hang.union(break_all)))
    }

    #[test]
    fn hang_lambda2() {
        let from = chain![
            chain!["let", BoxDoc::line(), "x", BoxDoc::line(), "="].group(),
            BoxDoc::line(),
            "\\y ->",
        ]
        .group();

        let body = chain!["y"].group();

        let trailer = BoxDoc::nil();

        let doc = hang2(from, BoxDoc::line(), body, trailer);
        eprintln!("{:#?}", doc);

        test!(doc, "let x = \\y -> y");
        test!(14, doc, "let x = \\y ->\n  y");
    }

    #[test]
    fn union2() {
        let from = chain![
            chain!["let", BoxDoc::line(), "x", BoxDoc::line(), "="].group(),
            BoxDoc::line(),
            "(",
        ]
        .group();

        let body = chain![
            chain!["x", ","].group(),
            BoxDoc::line(),
            chain!["1234567890", ","].group()
        ]
        .group();

        let trailer = BoxDoc::line_().append(")");

        let doc = hang2(from, BoxDoc::line_(), body, trailer);

        test!(doc, "let x = (x, 1234567890,)");
        test!(14, doc, "let x = (\n  x,\n  1234567890,\n)");
    }

    #[test]
    fn usize_max_value() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("test")
                .append(BoxDoc::line())
                .append(BoxDoc::text("test")),
        );

        test!(usize::MAX, doc, "test test");
    }

    #[test]
    fn fail() {
        let fail_break: BoxDoc<()> = BoxDoc::fail().flat_alt(Doc::nil());

        let doc = fail_break.append(Doc::text("12345")).group().union("abc");

        test!(5, doc, "12345");
        test!(4, doc, "abc");
    }

    pub struct TestWriter<W> {
        upstream: W,
    }

    impl<W> TestWriter<W> {
        pub fn new(upstream: W) -> Self {
            Self { upstream }
        }
    }

    impl<W> Render for TestWriter<W>
    where
        W: Render,
    {
        type Error = W::Error;

        fn write_str(&mut self, s: &str) -> Result<usize, W::Error> {
            self.upstream.write_str(s)
        }

        fn write_str_all(&mut self, s: &str) -> Result<(), W::Error> {
            self.upstream.write_str_all(s)
        }

        fn fail_doc(&self) -> Self::Error {
            self.upstream.fail_doc()
        }
    }

    impl<W> RenderAnnotated<'_, ()> for TestWriter<W>
    where
        W: Render,
    {
        fn push_annotation(&mut self, _: &()) -> Result<(), Self::Error> {
            self.upstream.write_str_all("[")
        }

        fn pop_annotation(&mut self) -> Result<(), Self::Error> {
            self.upstream.write_str_all("]")
        }
    }

    #[test]
    fn annotations() {
        let actual = BoxDoc::text("abc").annotate(()).annotate(());
        let mut s = String::new();
        actual
            .render_raw(70, &mut TestWriter::new(FmtWrite::new(&mut s)))
            .unwrap();
        difference::assert_diff!(&s, "[[abc]]", "\n", 0);
    }

    #[test]
    fn non_ascii_is_not_byte_length() {
        let doc: BoxDoc<()> = BoxDoc::group(
            BoxDoc::text("ÅÄÖ")
                .append(BoxDoc::line())
                .append(BoxDoc::text("test")),
        );

        test!(8, doc, "ÅÄÖ test");
    }

    #[test]
    fn cjk_display_width() {
        let arena = Arena::<()>::new();
        let doc = arena
            .text("你好")
            .append(arena.line().append(arena.text("abc")).align())
            .into_doc();

        test!(doc, "你好\n    abc");
    }

    #[test]
    fn pretty_cow() {
        let doc: BoxDoc<()> = docs![
            &BoxAllocator,
            Cow::<str>::Borrowed("abc"),
            BoxDoc::line(),
            Cow::<str>::Owned("123".to_string()),
        ]
        .group()
        .into_doc();

        test!(8, doc, "abc 123");
    }

    #[test]
    fn stress_append_left_assoc() {
        let arena = Arena::new();
        let mut doc: DocBuilder<'_, Arena<'_, _>, char> = arena.nil();
        for _ in 0..100000 {
            doc = doc.append("a");
        }
        let mut s = String::new();
        doc.render_fmt(80, &mut s).unwrap();
    }

    #[test]
    fn stress_append_right_assoc() {
        let arena = Arena::new();
        let mut doc: RefDoc<'_, char> = arena.nil().into_doc();
        for _ in 0..100000 {
            doc = arena.text("a").append(doc).into_doc();
        }
        let mut s = String::new();
        doc.render_fmt(80, &mut s).unwrap();
    }
}
