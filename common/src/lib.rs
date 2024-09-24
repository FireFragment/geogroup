use core::fmt;

#[derive(Debug, Clone)]
pub enum HiearchyItem<F> {
    Group(Vec<HiearchyItem<F>>),
    Item(F),
}

impl<I> HiearchyItem<I> {
    pub fn map_leafs<Out, Fun: Fn(I) -> Out>(self, fun: &Fun) -> HiearchyItem<Out> {
        match self {
            HiearchyItem::Group(g) => {
                HiearchyItem::Group(g.into_iter().map(|item| item.map_leafs(fun)).collect())
            }
            HiearchyItem::Item(it) => HiearchyItem::Item(fun(it)),
        }
    }

    pub fn leaves_mut<'s>(&'s mut self) -> Box<dyn Iterator<Item = &mut I> + 's> {
        match self {
            HiearchyItem::Group(g) => Box::new(g.iter_mut().flat_map(|item| item.leaves_mut())),
            HiearchyItem::Item(it) => Box::new(std::iter::once(it)),
        }
    }
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

impl<F: fmt::Display, D> HiearchyItem<(F, D)> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree_points_only(&self) -> String {
        match self {
            Self::Group(group) => format!(
                "--*\n{}",
                group
                    .iter()
                    .map(|subitem| subitem.print_tree_points_only().replace("\n", "\n  |"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Self::Item(item) => format!("--{}", item.0),
        }
    }
}
