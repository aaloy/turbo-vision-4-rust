//! A tree that expands and collapses. Up and Down move, Right opens a
//! branch, Left closes it, Enter toggles it.
//!
//! Parameters:
//! - `OutlineViewer::new(bounds, format)`: `format` is a closure that turns
//!   a node's value into the text shown.
//! - `Node::new(value)`, `Node::with_children(value, children)`: the tree's
//!   nodes, holding values of any type, each in an `Rc<RefCell<..>>`;
//!   `add_child` grows a branch later.
//! - `set_roots(nodes)`, `add_root(node)`: the top-level nodes.
//! - `selected_node()`: the node under the bar.
//!
//! See also: ListBox, Table, TextViewer

use crate::panel::Panel;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::outline::{Node, OutlineViewer};

type Tree = Rc<RefCell<Node<&'static str>>>;

pub fn build(panel: &mut Panel) {
    let mut tree = OutlineViewer::new(Rect::new(0, 0, 40, 7), |name: &&str| (*name).to_string());
    tree.set_roots(vec![
        branch(
            "src",
            vec![
                leaf("main.rs"),
                branch("views", vec![leaf("button.rs"), leaf("table.rs")]),
            ],
        ),
        branch("docs", vec![leaf("FORMS.md"), leaf("AGENTS.md")]),
        leaf("Cargo.toml"),
    ]);
    panel.add(tree);
}

fn leaf(name: &'static str) -> Tree {
    Rc::new(RefCell::new(Node::new(name)))
}

fn branch(name: &'static str, children: Vec<Tree>) -> Tree {
    Rc::new(RefCell::new(Node::with_children(name, children)))
}
