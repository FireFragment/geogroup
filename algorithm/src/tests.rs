use core::fmt;

use super::*;

/// Simple test item that implements SortableItem
#[derive(Debug, Clone, PartialEq)]
struct TestItem {
    pub time: u64,
    pub position: u64,
    pub id: String,
}

impl SortableItem for TestItem {
    type Time = u64;
    type Position = u64;
    type PositionErr = Infallible;

    fn get_time(&self) -> Self::Time {
        self.time
    }

    fn get_position(&self) -> Result<Self::Position, Infallible> {
        Ok(self.position)
    }
}

// #[test]
fn geogroup_algo_test() {
    let sorter = Sorter::<_, ()>::new(
        vec![
            TestItem {
                time: 1,
                position: 0,
                id: String::from("1"),
            },
            TestItem {
                time: 1,
                position: 2,
                id: String::from("1"),
            },
            TestItem {
                time: 1,
                position: 3,
                id: String::from("1"),
            },
            TestItem {
                time: 1,
                position: 8,
                id: String::from("1"),
            },
            TestItem {
                time: 1,
                position: 9,
                id: String::from("1"),
            },
            TestItem {
                time: 1,
                position: 14,
                id: String::from("1"),
            },
        ],
        Params { depth: MAX_DEPTH, ..Default::default() },
    );

    sorter.debug();

    panic!()
}
