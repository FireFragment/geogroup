use core::fmt;

use either::Either;

#[derive(Debug, Clone)]
pub enum HiearchyItem<LeafData, GroupData = ()> {
    Group(Vec<HiearchyItem<LeafData, GroupData>>, GroupData),
    Item(LeafData),
}

impl<L, G> HiearchyItem<L, G> {
    pub fn map_leafs<Out, Fun: Fn(L) -> Out>(self, fun: &Fun) -> HiearchyItem<Out, G> {
        match self {
            HiearchyItem::Group(group, data) => HiearchyItem::Group(
                group.into_iter().map(|item| item.map_leafs(fun)).collect(),
                data,
            ),
            HiearchyItem::Item(it) => HiearchyItem::Item(fun(it)),
        }
    }

    pub fn map_group_data<Out, Fun: Fn(G) -> Out>(self, fun: &Fun) -> HiearchyItem<L, Out> {
        match self {
            HiearchyItem::Group(group, data) => HiearchyItem::Group(
                group
                    .into_iter()
                    .map(|item| item.map_group_data(fun))
                    .collect(),
                fun(data),
            ),
            HiearchyItem::Item(it) => HiearchyItem::Item(it),
        }
    }

    pub fn leaves_mut<'s>(&'s mut self) -> Box<dyn Iterator<Item = &mut L> + 's> {
        match self {
            HiearchyItem::Group(g, _) => Box::new(g.iter_mut().flat_map(|item| item.leaves_mut())),
            HiearchyItem::Item(it) => Box::new(std::iter::once(it)),
        }
    }

    pub fn leaves<'s>(&'s self) -> Box<dyn Iterator<Item = &L> + 's> {
        match self {
            HiearchyItem::Group(g, _) => Box::new(g.iter().flat_map(|item| item.leaves())),
            HiearchyItem::Item(it) => Box::new(std::iter::once(it)),
        }
    }


    pub fn leaves_cloned<'s>(&'s self) -> impl Iterator<Item = L> where L: Clone {
        match self {
            HiearchyItem::Group(g, _) => g.iter().map(|item| item.leaves_cloned()).flatten().collect::<Vec<_>>().into_iter(),
            HiearchyItem::Item(it) => vec![it.to_owned()].into_iter(),
        }
    }
}

impl<L: fmt::Display, G: fmt::Debug> HiearchyItem<L, G> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree(&self) -> String {
        match self {
            Self::Group(group, data) => format!(
                "--* {data:?}\n{}",
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

impl<F: fmt::Display, D, G: fmt::Debug> HiearchyItem<(F, D), G> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree_points_only(&self) -> String {
        match self {
            Self::Group(group, data) => format!(
                "--* {data:?}\n{}",
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
