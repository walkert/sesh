extern crate skim;
use skim::prelude::*;
use std::io::Cursor;

pub fn get_choice(input: String) -> Option<Vec<String>> {
    let options = SkimOptionsBuilder::default()
        .reverse(true)
        .header(Some("Select directory or session"))
        .no_clear_start(true)
        .multi(true)
        .build()
        .unwrap();

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // Get either the selected items as a Vec of Strings or the query that was issued if no matches
    let selected_items = Skim::run_with(&options, Some(items)).map(|out| {
        if out.is_abort {
            return None;
        }
        if out.selected_items.len() > 0 {
            Some(
                out.selected_items
                    .into_iter()
                    .map(|item| item.output().into_owned())
                    .collect(),
            )
        } else {
            Some(vec![out.query])
        }
    });
    selected_items.expect("Unable to extract any values from Skim!")
}
