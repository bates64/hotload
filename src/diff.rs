use crate::program::{Item, Program};

#[derive(Debug, Clone)]
pub enum Diff<'old, 'new> {
    Add(&'new Item<'new>),
    Remove(&'old Item<'old>),
    Change(&'old Item<'old>, &'new Item<'new>),
}

/// Finds the difference between two [Programs](crate::program::Program).
pub fn diff<'old: 'new, 'new>(
    old: &'old Program<'old>,
    new: &'new Program<'new>,
) -> Vec<Diff<'old, 'new>> {
    let mut diffs = Vec::new();

    // Find items that were removed
    for (name, old_item) in &old.items {
        if !new.items.contains_key(name) {
            diffs.push(Diff::Remove(old_item));
        }
    }

    // Find items that were added
    for (name, new_item) in &new.items {
        if !old.items.contains_key(name) {
            diffs.push(Diff::Add(new_item));
        }
    }

    // Find items that were changed
    for (name, old_item) in &old.items {
        if let Some(new_item) = new.items.get(name) {
            if old_item != new_item {
                diffs.push(Diff::Change(old_item, new_item));
            }
        }
    }

    diffs
}
