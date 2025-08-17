use zellij_tile::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use crate::filter;

#[derive(Debug, Default)]
pub struct DirList {
    unique: HashSet<String>,
    dirs: Vec<String>,
    cursor: usize,

    search_term: String,
    filtered_dirs: Vec<String>,
}

impl DirList {
    pub fn reset(&mut self) {
        self.dirs.clear();
        self.cursor = 0;
        self.filtered_dirs.clear();
    }

    pub fn update_dirs(&mut self, dirs: Vec<String>) {
        dirs.iter().for_each(|dir| {
            if !self.unique.contains(dir) {
                self.unique.insert(dir.clone());
                self.dirs.push(dir.clone());
            }
        });
        self.dirs.sort_by(|a, b| b.cmp(a));
        self.cursor = 0;
        self.filter();
    }

    pub fn handle_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn handle_down(&mut self) {
        if self.cursor < self.filtered_dirs.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        if self.cursor < self.filtered_dirs.len() {
            Some(self.filtered_dirs[self.cursor].clone())
        } else {
            None
        }
    }

    pub fn set_search_term(&mut self, search_term: &str) {
        self.search_term = search_term.to_string();
        self.filter();
    }

    pub fn filter(&mut self) {
        self.filtered_dirs = filter::fuzzy_filter(&self.dirs, self.search_term.as_str());
        self.cursor = 0;
    }

    pub fn render(&self, rows: usize, _cols: usize, sessions: &HashMap<String, (bool, usize)>) {
        let mut sorted_dirs = self.filtered_dirs.clone();
        sorted_dirs.sort_by(|a, b| {
            let a_folder = Path::new(a)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let b_folder = Path::new(b)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            let a_has_session = sessions.contains_key(a_folder);
            let b_has_session = sessions.contains_key(b_folder);

            match (a_has_session, b_has_session) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });

        let max_display_rows = rows.saturating_sub(4);
        let from = self
            .cursor
            .saturating_sub(max_display_rows.saturating_sub(1) / 2)
            .min(sorted_dirs.len().saturating_sub(max_display_rows));

        let mut folder_names = HashSet::new();
        let mut duplicates = HashSet::new();
        for dir in &sorted_dirs {
            let folder_name = Path::new(dir)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !folder_names.insert(folder_name) {
                duplicates.insert(folder_name);
            }
        }

        let total_remaining = sorted_dirs.len().saturating_sub(from);
        let items_to_show = max_display_rows.min(total_remaining);

        sorted_dirs
            .iter()
            .enumerate()
            .skip(from)
            .take(items_to_show)
            .for_each(|(i, dir)| {
                let folder_name = Path::new(dir)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                let (base_text, user_count_start, user_count_end) =
                    if let Some((is_current, connected_users)) = sessions.get(folder_name) {
                        let base = if duplicates.contains(folder_name) {
                            format!("       > {} ({})", folder_name, dir)
                        } else {
                            format!("       > {}", folder_name)
                        };

                        if *is_current {
                            let full_text =
                                format!("{} [CURRENT - {} users]", base, connected_users);
                            let user_start = base.len() + " [CURRENT - ".len();
                            let user_end = user_start + connected_users.to_string().len();
                            (full_text, Some(user_start), Some(user_end))
                        } else if *connected_users > 0 {
                            let full_text = format!("{} [{} users]", base, connected_users);
                            let user_start = base.len() + " [".len();
                            let user_end = user_start + connected_users.to_string().len();
                            (full_text, Some(user_start), Some(user_end))
                        } else {
                            let full_text = format!("{} [INACTIVE]", base);
                            let inactive_start = base.len() + " [".len();
                            let inactive_end = inactive_start + "INACTIVE".len();
                            (full_text, Some(inactive_start), Some(inactive_end))
                        }
                    } else {
                        let base = if duplicates.contains(folder_name) {
                            format!("       > {} ({})", folder_name, dir)
                        } else {
                            format!("       > {}", folder_name)
                        };
                        (format!("{} [NOT CREATED]", base), None, None)
                    };

                let text_len = base_text.len();
                let mut item = Text::new(&base_text);

                if let (Some(start), Some(end)) = (user_count_start, user_count_end) {
                    let is_inactive = base_text.contains("[INACTIVE]");
                    let color = if is_inactive { 1 } else { 3 };
                    item = item.color_range(color, start..end);
                }
                let item = match i == self.cursor {
                    true => item.color_range(0, 0..text_len).selected(),
                    false => item,
                };
                print_text(item);
                println!();
            });

        let displayed_items = items_to_show;
        let remaining_items = sorted_dirs.len().saturating_sub(from + displayed_items);
        let all_items_fit = sorted_dirs.len() <= max_display_rows;
        if remaining_items > 0 && !all_items_fit {
            let more_text = format!("       +{} more", remaining_items);
            let item = Text::new(&more_text).color_range(2, 0..more_text.len());
            print_text(item);
            println!();
        }
    }
}
