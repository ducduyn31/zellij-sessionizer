use zellij_tile::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use crate::filter;
use crate::utils::{format_base_text, format_duration, get_folder_name};

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

    pub fn get_selected(
        &self,
        sessions: &HashMap<String, (bool, usize)>,
        resurrectable_sessions: &HashMap<String, Duration>,
    ) -> Option<String> {
        let sorted_dirs = self.get_sorted_dirs_with_sessions(sessions, resurrectable_sessions);
        if self.cursor < sorted_dirs.len() {
            Some(sorted_dirs[self.cursor].clone())
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

    fn get_sorted_dirs_with_sessions(
        &self,
        sessions: &HashMap<String, (bool, usize)>,
        resurrectable_sessions: &HashMap<String, Duration>,
    ) -> Vec<String> {
        let mut sorted_dirs = self.filtered_dirs.clone();
        sorted_dirs.sort_by(|a, b| {
            let a_folder = get_folder_name(a);
            let b_folder = get_folder_name(b);

            let a_has_session = sessions.contains_key(a_folder);
            let b_has_session = sessions.contains_key(b_folder);
            let a_has_resurrectable = resurrectable_sessions.contains_key(a_folder);
            let b_has_resurrectable = resurrectable_sessions.contains_key(b_folder);

            match (
                a_has_session,
                b_has_session,
                a_has_resurrectable,
                b_has_resurrectable,
            ) {
                (true, false, _, _) => std::cmp::Ordering::Less,
                (false, true, _, _) => std::cmp::Ordering::Greater,
                (false, false, true, false) => std::cmp::Ordering::Less,
                (false, false, false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });
        sorted_dirs
    }

    pub fn render(
        &self,
        rows: usize,
        _cols: usize,
        sessions: &HashMap<String, (bool, usize)>,
        resurrectable_sessions: &HashMap<String, Duration>,
    ) {
        let sorted_dirs = self.get_sorted_dirs_with_sessions(sessions, resurrectable_sessions);

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
                let folder_name = get_folder_name(dir);
                let base = format_base_text(folder_name, dir, &duplicates);

                let (base_text, user_count_start, user_count_end) =
                    if let Some((is_current, connected_users)) = sessions.get(folder_name) {
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
                            let full_text = format!("{} [CREATED]", base);
                            let created_start = base.len() + " [".len();
                            let created_end = created_start + "CREATED".len();
                            (full_text, Some(created_start), Some(created_end))
                        }
                    } else if let Some(creation_time) = resurrectable_sessions.get(folder_name) {
                        let time_str = format_duration(*creation_time);
                        let full_text = format!("{} [EXITED {}]", base, time_str);
                        let exited_start = base.len() + " [".len();
                        let exited_end = exited_start + "EXITED".len();
                        (full_text, Some(exited_start), Some(exited_end))
                    } else {
                        (format!("{} [NOT CREATED]", base), None, None)
                    };

                let text_len = base_text.len();
                let mut item = Text::new(&base_text);

                if let (Some(start), Some(end)) = (user_count_start, user_count_end) {
                    let is_created = base_text.contains("[CREATED]");
                    let is_exited = base_text.contains("[EXITED]");
                    let color = if is_created || is_exited { 1 } else { 3 };
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
