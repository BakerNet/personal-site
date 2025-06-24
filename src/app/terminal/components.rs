use leptos::prelude::*;

const CHAR_WIDTH: usize = 9;
const TERMINAL_MARGINS: usize = 65;

fn num_rows(num_items: usize, cols: usize) -> usize {
    let items_per_row = num_items / cols;
    if num_items % cols > 0 {
        items_per_row + 1
    } else {
        items_per_row
    }
}

pub trait TextContent {
    fn text_content(&self) -> &str;
}

impl TextContent for String {
    fn text_content(&self) -> &str {
        self.as_str()
    }
}

#[component]
pub fn ColumnarView<T, F>(items: Vec<T>, render_func: F) -> impl IntoView
where
    T: TextContent + Clone + 'static,
    F: Fn(T) -> AnyView + 'static,
{
    let available_space = window()
        .inner_width()
        .expect("should be able to get window width")
        .as_f64()
        .expect("window width should be a number")
        .round() as usize
        - TERMINAL_MARGINS;
    let available_space = available_space / CHAR_WIDTH;
    let total_len = items
        .iter()
        .map(|s| s.text_content().len() + 2)
        .sum::<usize>();
    if total_len < available_space {
        return view! {
            {items
                .into_iter()
                .map(|s| {
                    view! {
                        {render_func(s)}
                        "  "
                    }
                })
                .collect_view()}
        }
        .into_any();
    }
    let max_cols = 10.min(items.len());
    let mut cols = 1;
    for n in 0..max_cols {
        let n = max_cols - n;
        let per_col = num_rows(items.len(), n);
        let total_len = items
            .chunks(per_col)
            .map(|v| {
                v.iter()
                    .map(|s| s.text_content().len() + 2)
                    .max()
                    .expect("there should be a max len for each column")
            })
            .sum::<usize>();
        if total_len < available_space {
            cols = n;
            break;
        }
    }
    let rows = num_rows(items.len(), cols);
    let item_cols = items
        .chunks(rows)
        .map(|x| x.to_vec())
        .collect::<Vec<Vec<T>>>();
    let col_lens = item_cols
        .iter()
        .map(|v| {
            v.iter()
                .map(|s| s.text_content().len() + 2)
                .max()
                .expect("there should be a max len for each column")
        })
        .collect::<Vec<_>>();
    let views = (0..rows)
        .map(|row| {
            view! {
                <div>
                    {item_cols
                        .iter()
                        .zip(col_lens.iter())
                        .filter(|(v, _)| row < v.len())
                        .map(|(v, l)| (&v[row], l))
                        .map(|(s, l)| {
                            let fill = l - s.text_content().len();
                            view! {
                                {render_func(s.clone())}
                                {" ".repeat(fill)}
                            }
                        })
                        .collect_view()}
                </div>
            }
        })
        .collect::<Vec<_>>();
    view! { {views} }.into_any()
}
