//! This is a fork of `https://raw.githubusercontent.com/sagebind/smplinfo/master/src/ui/widgets/table.rs` with
//! some modifications.

use std::ops::Range;
use std::{fmt::Display, hash::Hash};

use eframe::egui::{vec2, Id, Response, ScrollArea, Sense, TextStyle, Ui, Vec2, Widget};

const DEFAULT_COLUMN_WIDTH: f32 = 200.0;

/// An ordinary table. Feature set is currently pretty limited.
///
/// - `R`: The data type of a single row displayed.
/// - `C`: The type of collection holding the rows to display. Any collection
///   implementing `AsRef<[R]>` can be used.
pub struct Table<
    'selection,
    R,
    C: AsRef<[Option<R>]>,
    RowMaker: FnMut(Range<usize>) -> C,
    RowSelection: Fn(&Option<R>),
> {
    id_source: Id,
    columns: Vec<Column<R>>,
    selected_row: Option<&'selection mut Option<usize>>,
    header_height: f32,
    row_height: f32,
    cell_padding: Vec2,
    row_maker: RowMaker,
    num_rows: usize,
    pub row_action: Option<RowSelection>,
}

/// Table column definition.
struct Column<R> {
    name: String,
    value_mapper: Box<dyn FnMut(&Option<R>) -> String>,
    max_width: Option<f32>,
}

impl<
        R,
        C: AsRef<[Option<R>]>,
        RowMaker: FnMut(Range<usize>) -> C,
        RowSelection: Fn(&Option<R>),
    > Table<'static, R, C, RowMaker, RowSelection>
{
    #[allow(dead_code)]
    pub fn new(id_source: impl Hash, num_rows: usize, row_maker: RowMaker) -> Self {
        Self {
            id_source: Id::new(id_source),
            columns: Vec::new(),
            selected_row: None,
            header_height: 28.0,
            row_height: 24.0,
            cell_padding: vec2(8.0, 4.0),
            row_maker,
            num_rows,
            row_action: None,
        }
    }
}

impl<
        's,
        R,
        C: AsRef<[Option<R>]>,
        RowMaker: FnMut(Range<usize>) -> C,
        RowSelection: Fn(&Option<R>),
    > Table<'s, R, C, RowMaker, RowSelection>
{
    pub fn new_selectable(
        id_source: impl Hash,
        selected_row: &'s mut Option<usize>,
        num_rows: usize,
        selection: RowSelection,
        row_maker: RowMaker,
    ) -> Self {
        Self {
            id_source: Id::new(id_source),
            columns: Vec::new(),
            selected_row: Some(selected_row),
            header_height: 28.0,
            row_height: 24.0,
            cell_padding: vec2(8.0, 4.0),
            row_maker,
            num_rows,
            row_action: Some(selection),
        }
    }

    pub fn column(
        mut self,
        name: impl Display,
        width: f32,
        value_mapper: impl FnMut(&Option<R>) -> String + 'static,
    ) -> Self {
        self.columns.push(Column {
            name: name.to_string(),
            value_mapper: Box::new(value_mapper),
            max_width: Some(width),
        });
        self
    }

    #[allow(dead_code)]
    fn supports_selection(&self) -> bool {
        self.selected_row.is_some()
    }

    fn header_ui(&mut self, ui: &mut Ui, state: &mut State) {
        let header_text_style = TextStyle::Body;

        // Table always grows as wide as available, so the header should too.
        let (_, rect) = ui.allocate_space(vec2(ui.available_width(), self.header_height));

        // Header background
        let painter = ui.painter_at(rect);
        painter.rect_filled(
            rect,
            ui.visuals().widgets.inactive.corner_radius,
            ui.visuals().widgets.inactive.bg_fill,
        );

        let mut column_offset = 0.0;
        let column_len = self.columns.len();

        for (i, column) in self.columns.iter().enumerate() {
            let column_id = self.id_source.with("_column_").with(i);

            let desired_column_width = state.column_width(i);
            let galley = ui
                .fonts()
                .layout_single_line(header_text_style, column.name.clone());

            let mut column_rect = rect;
            column_rect.min.x += column_offset;

            let is_last = i == (column_len - 1);
            if !is_last && column_rect.width() > desired_column_width {
                column_rect.set_width(desired_column_width);
            }

            let response = ui.interact(column_rect, column_id, Sense::hover());

            if response.hovered() {
                ui.painter().rect_stroke(
                    column_rect,
                    ui.visuals().widgets.hovered.corner_radius,
                    ui.visuals().widgets.hovered.bg_stroke,
                );
            }

            let mut text_pos = column_rect.left_center();
            text_pos.x += self.cell_padding.x;
            text_pos.y -= galley.size.y / 2.0;
            ui.painter_at(column_rect).galley(
                text_pos,
                galley,
                if response.hovered() {
                    ui.style().visuals.widgets.hovered.fg_stroke.color
                } else {
                    ui.style().visuals.widgets.inactive.fg_stroke.color
                },
            );

            column_offset += column_rect.width();
        }
    }
}

impl<
        's,
        R: std::fmt::Debug,
        C: AsRef<[Option<R>]>,
        RowMaker: FnMut(Range<usize>) -> C,
        RowAction: Fn(&Option<R>),
    > Widget for Table<'s, R, C, RowMaker, RowAction>
{
    fn ui(mut self, ui: &mut Ui) -> Response {
        if self.columns.is_empty() {
            panic!("uh, what do I do if no columns are defined?");
        }

        let mut state = ui
            .memory()
            .id_data_temp
            .get_or_default::<State>(self.id_source)
            .clone();

        // set the sizes on the state
        state.column_widths = self
            .columns
            .iter()
            .map(|e| e.max_width.unwrap_or(DEFAULT_COLUMN_WIDTH))
            .collect();

        // First step: compute some sizes used during rendering. Since this is a
        // homogenous table, we can figure out its exact sizes based on the
        // number of rows and columns.
        let table_rect = ui.available_rect_before_wrap_finite();
        let response = ui.interact(table_rect, self.id_source, Sense::hover());

        self.header_ui(ui, &mut state);

        // Now render the table body, which is inside an independently
        // scrollable area.
        ScrollArea::auto_sized().show_rows(ui, self.row_height, self.num_rows, |ui, row_range| {
            ui.scope(|ui| {
                let maker = &mut self.row_maker;
                let rows = maker(row_range);

                // When laying out the table, don't allocate any spacing between the
                // rows.
                ui.spacing_mut().item_spacing.y = 0.0;

                // TODO: Decide row height more intelligently...
                let row_size = vec2(ui.available_width(), self.row_height);
                let cell_text_style = ui.style().body_text_style;
                let column_len = self.columns.len();

                for (row_idx, row) in rows.as_ref().iter().enumerate() {
                    let (row_id, row_rect) = ui.allocate_space(row_size);
                    let mut row_response = ui.interact(row_rect, row_id, Sense::click());
                    let mut cell_text_color = ui.style().visuals.text_color();

                    // If this row is currently selected, make it look like it is.
                    if self.selected_row == Some(&mut Some(row_idx)) {
                        cell_text_color = ui.visuals().strong_text_color();
                        ui.painter().rect_filled(
                            row_rect,
                            0.0,
                            ui.style().visuals.selection.bg_fill,
                        );
                    } else if row_response.hovered() {
                        ui.painter().rect_filled(
                            row_rect,
                            0.0,
                            ui.visuals().widgets.hovered.bg_fill,
                        );
                    } else if row_idx % 2 > 0 {
                        ui.painter()
                            .rect_filled(row_rect, 0.0, ui.visuals().faint_bg_color);
                    }

                    // Give the hovered mails a tooltip
                    if row_response.hovered() {
                        //let data = self.columns[row_id];
                        let mut hover_data = Vec::new();
                        for (_, column) in self.columns.iter_mut().enumerate() {
                            let cell_text = (column.value_mapper)(row);
                            hover_data.push(cell_text);
                        }
                        let hover_string = hover_data.join("\n");
                        row_response = row_response.on_hover_text(hover_string);
                    }

                    let mut column_offset = 0.0;

                    for (col_idx, column) in self.columns.iter_mut().enumerate() {
                        let desired_column_width = state.column_width(col_idx);
                        let cell_text = (column.value_mapper)(row);

                        let mut column_rect = row_rect;
                        column_rect.min.x += column_offset;

                        // Auto-expand the last column
                        let is_last = col_idx == (column_len - 1);
                        if !is_last && column_rect.width() > desired_column_width {
                            column_rect.set_width(desired_column_width);
                        }

                        let painter = ui.painter_at(column_rect);

                        let galley = ui.fonts().layout_single_line(cell_text_style, cell_text);

                        let mut text_pos = column_rect.left_center();
                        text_pos.x += self.cell_padding.x;
                        text_pos.y -= galley.size.y / 2.0;
                        painter.galley(text_pos, galley, cell_text_color);

                        column_offset += column_rect.width();
                    }

                    if let Some(selected_row) = self.selected_row.as_mut() {
                        if row_response.clicked() {
                            if let Some(ref n) = self.row_action {
                                if let Some(a) = rows.as_ref().get(row_idx) {
                                    n(a)
                                }
                            }
                            **selected_row = Some(row_idx);
                        }
                    }
                }
            });
        });

        response
    }
}

/// Persistent table UI state.
#[derive(Clone, Default)]
struct State {
    /// Current width of each column. This is updated when a column is resized.
    column_widths: Vec<f32>,
}

impl State {
    fn column_width(&self, column: usize) -> f32 {
        self.column_widths
            .get(column)
            .cloned()
            .unwrap_or(DEFAULT_COLUMN_WIDTH)
    }
}
