//! Rows and columns under a header. The arrows move the focused cell, PgUp
//! and PgDn scroll, Home and End go to the first and last row. Enter or a
//! double click sends the table's command. Here the columns are wider than
//! the table: move right and Product, frozen, stays at the left edge.
//!
//! Parameters:
//! - `Table::new(bounds, command)`: the header takes the top row; `command`
//!   is sent on Enter or a double click (0 for none).
//! - `Column::new(title, width)` and `Column::right(title, width)`: a
//!   column aligned left, for text, or right, for numbers; `width` leaves
//!   out the one-column gap to the next column. Give them with
//!   `set_columns`.
//! - `set_rows(rows)`, `add_row(row)`, `clear_rows()`: rows of strings, one
//!   per column.
//! - `set_frozen_cols(n)`: freeze the first `n` columns, so they stay at
//!   the left edge while the others scroll sideways. A line marks where the
//!   frozen columns end.
//! - `set_frozen_rows(n)`: freeze the first `n` rows under the header while
//!   the others scroll up and down; the last frozen row is underlined.
//!   Frozen rows are still focused and selected like any other.
//! - `set_show_header(false)`: no header row. `set_separators(true)`: a
//!   line between the columns.
//! - `set_provider(provider)`: read the rows from a `RowProvider` as they
//!   are drawn, from a database for instance; `refresh_rows()` after its
//!   length changes.
//! - `selected_row()`, `selected_col()`, `selected_cell()`: where the focus
//!   is; `set_selected_row(i)` and `set_selected_col(i)` move it.
//!
//! See also: ListBox, SortedListBox, Outline

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::table::{Column, Table};

pub fn build(panel: &mut Panel) {
    let mut table = Table::new(Rect::new(0, 0, 46, 6), 0);
    table.set_columns(vec![
        Column::new("Product", 12),
        Column::right("Units", 6),
        Column::right("Price", 8),
        Column::right("Total", 9),
        Column::new("Supplier", 12),
        Column::right("Stock", 6),
    ]);
    table.set_rows(vec![
        row("Keyboard", 3, 49.90, "Keys & Co", 40),
        row("Monitor", 1, 219.00, "Pixelworks", 6),
        row("Mouse", 5, 19.50, "Keys & Co", 85),
        row("Cable", 12, 4.25, "Wired Ltd", 300),
        row("Dock", 2, 129.00, "Pixelworks", 11),
    ]);
    // The columns are wider than the table: Right scrolls them sideways,
    // and Product, frozen, stays at the left edge.
    table.set_frozen_cols(1);
    panel.add(table);
}

fn row(product: &str, units: u32, price: f64, supplier: &str, stock: u32) -> Vec<String> {
    vec![
        product.to_string(),
        units.to_string(),
        format!("{price:.2}"),
        format!("{:.2}", f64::from(units) * price),
        supplier.to_string(),
        stock.to_string(),
    ]
}
