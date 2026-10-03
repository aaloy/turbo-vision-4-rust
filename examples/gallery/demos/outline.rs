//! A tree that expands and collapses. Up and Down move, Right opens a
//! branch, Left closes it, Enter toggles it. The tree is made of `Node`s
//! holding values of any type; the closure given to `new` turns a value
//! into the text shown. `selected_node()` returns the node under the bar.

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
