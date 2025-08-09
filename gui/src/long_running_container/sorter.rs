use super::*;
use std::thread;

pub struct SorterContainer<Item: backend::SortableItem>(Inner<Item>);
enum Inner<Item: backend::SortableItem> {
    Available(algorithm::Sorter<Item>),
    Constructing(thread::JoinHandle<backend::algorithm::Sorter<Item>>),
}

impl<Item: backend::SortableItem> SorterContainer<Item> {
    pub fn new() -> Self {
        SorterContainer(Inner::Constructing(thread::spawn(|| {})))
    }
}
