use std::collections::HashSet;

use egui::Id;

use crate::{
    pointer::JsonPointerSegment,
    value::{ExpandableType, JsonTreeValue, ToJsonTreeValue},
};

#[derive(Debug, Clone, Hash)]
pub struct SearchTerm(String);

impl SearchTerm {
    pub(crate) fn parse(search_str: &str) -> Option<Self> {
        Self::is_valid(search_str).then_some(Self(search_str.to_ascii_lowercase()))
    }

    const fn is_valid(search_str: &str) -> bool {
        !search_str.is_empty()
    }

    pub(crate) fn find_match_indices_in(&self, other: &str) -> Vec<usize> {
        other
            .to_ascii_lowercase()
            .match_indices(&self.0)
            .map(|(idx, _)| idx)
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn find_matching_paths_in<T: ToJsonTreeValue>(
        &self,
        value: &T,
        abbreviate_root: bool,
        make_persistent_id: &dyn Fn(&[JsonPointerSegment<'_>]) -> Id,
        reset_path_ids: &mut HashSet<Id>,
    ) -> HashSet<Id> {
        let mut search_match_path_ids = HashSet::new();

        search_impl(
            value,
            self,
            &mut vec![],
            &mut search_match_path_ids,
            make_persistent_id,
            reset_path_ids,
        );

        if !abbreviate_root && search_match_path_ids.len() == 1 {
            // The only match was a top level key or value - no need to expand anything.
            search_match_path_ids.clear();
        }

        search_match_path_ids
    }

    fn matches<V: ToString + ?Sized>(&self, other: &V) -> bool {
        other.to_string().to_ascii_lowercase().contains(&self.0)
    }
}

fn search_impl<'a, T: ToJsonTreeValue>(
    value: &'a T,
    search_term: &SearchTerm,
    path_segments: &mut Vec<JsonPointerSegment<'a>>,
    search_match_path_ids: &mut HashSet<Id>,
    make_persistent_id: &dyn Fn(&[JsonPointerSegment<'_>]) -> Id,
    reset_path_ids: &mut HashSet<Id>,
) {
    match value.to_json_tree_value() {
        JsonTreeValue::Base(_, display_value, _) => {
            if search_term.matches(display_value) {
                update_matches(path_segments, search_match_path_ids, make_persistent_id);
            }
        }
        JsonTreeValue::Expandable(entries, expandable_type) => {
            for (property, val) in &entries {
                path_segments.push(*property);

                if val.is_expandable() {
                    reset_path_ids.insert(make_persistent_id(path_segments));
                }

                // Ignore matches for indices in an array.
                if expandable_type == ExpandableType::Object && search_term.matches(property) {
                    update_matches(path_segments, search_match_path_ids, make_persistent_id);
                }

                search_impl(
                    *val,
                    search_term,
                    path_segments,
                    search_match_path_ids,
                    make_persistent_id,
                    reset_path_ids,
                );
                path_segments.pop();
            }
        }
    };
}

fn update_matches(
    path_segments: &[JsonPointerSegment<'_>],
    search_match_path_ids: &mut HashSet<Id>,
    make_persistent_id: &dyn Fn(&[JsonPointerSegment<'_>]) -> Id,
) {
    for i in 0..path_segments.len() {
        search_match_path_ids.insert(make_persistent_id(&path_segments[0..i]));
    }
}
