use core::fmt;

pub type Distance = u64;

#[derive(Debug, Clone)]
pub enum HiearchyItem<F> {
    Group(Vec<HiearchyItem<F>>),
    Item(F),
}

impl<F: fmt::Display> HiearchyItem<F> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree(&self) -> String {
        match self {
            Self::Group(group) => format!(
                "--*\n{}",
                group
                    .iter()
                    .map(|subitem| subitem.print_tree().replace("\n", "\n  |"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Self::Item(item) => format!("--{}", item),
        }
    }
}
