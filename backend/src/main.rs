use geogroup_backend::*;
use lazy_hierarchy::NodeRef;

fn main() {
    let NodeRef::Group(g) = fs_hierarchy::new(".".into()).unwrap() else {
        panic!()
    };

    println!(
        "{}",
        g.format_as_tree(
            |node| node
                .node_data()
                .file_name()
                .map(|name| name.to_string_lossy().into())
                .unwrap_or_else(|| String::new()),
            true
        )
    )
}
