// (C) 2026 - Antoni Aloy
// Table Format Demo - colours and text styles on rows, columns and cells.
//
// A task list where the formatting says what the plain text cannot:
//
//   a column format (set_column_format) makes the Hours column bold;
//   a row format (set_row_format) paints a late task red, and a finished one
//   dim and struck through;
//   a cell format (set_cell_format) marks one status that needs attention.
//
// Formats stack column, row, cell: the later one wins on colours and the
// styles add up. The selected row keeps the selection colours and takes only
// the style, so the bar still reads as one bar. In a terminal a "font" is
// the set of Style flags: bold, italic, underline, reverse, dim, strikethrough.
//
// F2 switches the formats on and off (clear_formats), Alt-X quits.

use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CM_USER, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Style, TvColor};
use turbo_vision::core::state::Grow;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::table::{CellFormat, Column, Table, TableBuilder};
use turbo_vision::views::window::{Window, WindowBuilder};
use turbo_vision::views::{GroupLike, View, ViewId};

const CM_TOGGLE_FORMATS: CommandId = CM_USER;

const TASKS: [(&str, &str, &str, &str, u32); 8] = [
    ("Write release notes", "Ana", "Oct 20", "Late", 6),
    ("Fix table redraw", "Joan", "Oct 22", "Done", 3),
    ("Review PR 128", "Marta", "Oct 25", "Blocked", 2),
    ("Update the guide", "Ana", "Oct 28", "Open", 8),
    ("Gallery demo", "Joan", "Nov 02", "Open", 5),
    ("Tag 4.2", "Marta", "Nov 05", "Open", 1),
    ("Check the website", "Ana", "Oct 18", "Late", 4),
    ("Close old issues", "Joan", "Oct 15", "Done", 2),
];

/// Put the formats on a table whose rows are `TASKS`.
fn apply_formats(table: &mut Table) {
    // A whole column: the hours stand out.
    table.set_column_format(4, CellFormat::new().style(Style::BOLD));
    for (row, task) in TASKS.iter().enumerate() {
        match task.3 {
            // A whole row: late tasks in red, finished ones dim and struck.
            "Late" => table.set_row_format(row, CellFormat::new().fg(TvColor::LightRed)),
            "Done" => table.set_row_format(
                row,
                CellFormat::new().style(Style::DIM | Style::STRIKETHROUGH),
            ),
            _ => {}
        }
    }
    // One cell: a blocked task needs somebody's attention.
    table.set_cell_format(
        2,
        3,
        CellFormat::new()
            .fg(TvColor::Black)
            .bg(TvColor::Yellow)
            .style(Style::BOLD | Style::UNDERLINE),
    );
}

/// Answers F2 by switching the formats on and off.
struct FormatApp {
    window: ViewId,
    table: ViewId,
    on: bool,
}

impl AppHandler for FormatApp {
    fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
        if command != CM_TOGGLE_FORMATS {
            return false;
        }
        let Some(table) = app
            .desktop
            .child_by_id_mut(self.window)
            .and_then(|w| w.as_any_mut().downcast_mut::<Window>())
            .and_then(|w| w.child_by_id_mut(self.table))
            .and_then(|t| t.as_any_mut().downcast_mut::<Table>())
        else {
            return true;
        };
        self.on = !self.on;
        if self.on {
            apply_formats(table);
        } else {
            table.clear_formats();
        }
        true
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text("~F2~ Formats on/off")
                .key("F2")
                .command(CM_TOGGLE_FORMATS)
                .build(),
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ],
    ));

    let rows = TASKS
        .iter()
        .map(|(task, owner, due, status, hours)| {
            vec![
                (*task).to_string(),
                (*owner).to_string(),
                (*due).to_string(),
                (*status).to_string(),
                hours.to_string(),
            ]
        })
        .collect();

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(8, 3, 72, 17))
        .title("Tasks")
        .build();
    let mut hint = StaticText::new(
        Rect::new(1, 0, 60, 1),
        "Red: late. Struck: done. Yellow cell: blocked. Bold: hours.",
    );
    hint.set_grow_mode(Grow::HI_X);
    window.add(hint);

    let mut table = TableBuilder::new()
        .bounds(Rect::new(1, 2, 61, 11))
        .columns(vec![
            Column::new("Task", 22),
            Column::new("Owner", 8),
            Column::new("Due", 8),
            Column::new("Status", 9),
            Column::right("Hours", 5),
        ])
        .rows(rows)
        .separators(true)
        .build();
    apply_formats(&mut table);
    table.set_grow_mode(Grow::HI_X | Grow::HI_Y);
    let table = window.add(table);
    window.set_initial_focus();
    let window = app.desktop.add(window);

    app.run_with(&mut FormatApp {
        window,
        table,
        on: true,
    });
    Ok(())
}
