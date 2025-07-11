use prettyless::{BoxAllocator, DocAllocator, DocBuilder};
use std::io;
use std::str;

#[derive(Clone, Debug)]
pub struct Forest<'a>(&'a [Tree<'a>]);

impl<'a> Forest<'a> {
    fn new(forest: &'a [Tree<'a>]) -> Forest<'a> {
        Forest(forest)
    }

    fn nil() -> Forest<'a> {
        Forest(&[])
    }

    fn bracket<'b, D>(&'b self, allocator: &'b D) -> DocBuilder<'b, D>
    where
        D: DocAllocator<'b>,
        D::Doc: Clone,
    {
        if (self.0).is_empty() {
            allocator.nil()
        } else {
            allocator
                .text("[")
                .append(allocator.hardline().append(self.pretty(allocator)).nest(2))
                .append(allocator.hardline())
                .append(allocator.text("]"))
        }
    }

    fn pretty<'b, D>(&'b self, allocator: &'b D) -> DocBuilder<'b, D>
    where
        D: DocAllocator<'b>,
        D::Doc: Clone,
    {
        let forest = self.0;
        let separator = allocator.text(",").append(allocator.hardline());
        allocator.intersperse(forest.iter().map(|tree| tree.pretty(allocator)), separator)
    }
}

#[derive(Clone, Debug)]
pub struct Tree<'a> {
    node: String,
    forest: Forest<'a>,
}

impl<'a> Tree<'a> {
    pub fn node(node: &str) -> Tree<'a> {
        Tree {
            node: node.to_string(),
            forest: Forest::nil(),
        }
    }

    pub fn node_with_forest(node: &str, forest: &'a [Tree<'a>]) -> Tree<'a> {
        Tree {
            node: node.to_string(),
            forest: Forest::new(forest),
        }
    }

    pub fn pretty<'b, D>(&'b self, allocator: &'b D) -> DocBuilder<'b, D>
    where
        D: DocAllocator<'b>,
        D::Doc: Clone,
    {
        allocator
            .text(&self.node[..])
            .append((self.forest).bracket(allocator))
            .group()
    }
}

#[allow(dead_code)]
pub fn main() {
    let allocator = BoxAllocator;
    let bbbbbbs = [Tree::node("ccc"), Tree::node("dd")];
    let ffffs = [Tree::node("gg"), Tree::node("hhh"), Tree::node("ii")];
    let aaas = [
        Tree::node_with_forest("bbbbbb", &bbbbbbs),
        Tree::node("eee"),
        Tree::node_with_forest("ffff", &ffffs),
    ];
    let example = Tree::node_with_forest("aaaa", &aaas);

    let err_msg = "<buffer is not a utf-8 encoded string>";

    // try writing to stdout
    {
        print!("\nwriting to stdout directly:\n");
        let mut out = io::stdout();
        example.pretty(&allocator).1.render(70, &mut out)
        // try writing to memory
    }
    .and_then(|()| {
        print!("\nwriting to string then printing:\n");
        let mut mem = Vec::new();
        example
            .pretty(&allocator)
            .1
            .render(70, &mut mem)
            // print to console from memory
            .map(|()| {
                let res = str::from_utf8(&mem).unwrap_or(err_msg);
                println!("{res}")
            })
        // print an error if anything failed
    })
    .unwrap_or_else(|err| println!("error: {err}"));
}
