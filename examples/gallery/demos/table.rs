//! Rows and columns under a header. The arrows move the focused cell,
//! PgUp and PgDn scroll, Home and End go to the first and last row. Enter
//! sends the table's command; `selected_row()` and `selected_col()` say
//! where. Columns align left or right; the first rows and columns can be
//! frozen; a `RowProvider` supplies rows on demand from a database.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::table::{Column, Table};

pub fn build(panel: &mut Panel) {
    let mut table = Table::new(Rect::new(0, 0, 46, 6), 0);
    table.set_columns(vec![
        Column::new("Product", 16),
        Column::right("Units", 7),
        Column::right("Price", 9),
        Column::right("Total", 10),
    ]);
    table.set_rows(vec![
        row("Keyboard", 3, 49.90),
        row("Monitor", 1, 219.00),
        row("Mouse", 5, 19.50),
        row("Cable", 12, 4.25),
        row("Dock", 2, 129.00),
    ]);
    panel.add(table);
}

fn row(product: &str, units: u32, price: f64) -> Vec<String> {
    vec![
        product.to_string(),
        units.to_string(),
        format!("{price:.2}"),
        format!("{:.2}", f64::from(units) * price),
    ]
}
