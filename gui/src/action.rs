//! Actions are _large functions_ that are called by [GUI](gui)

use std::thread;

use super::*;

pub(crate) fn sort(
    operation_config: &mut geogroup_backend::SortingCfg,
    hiearchy: Vec<geogroup_backend::HiearchyItem<hiearchy::File, String>>,
    args: &CliArgs,
    inbox: &UiInbox<Message>,
) -> thread::JoinHandle<()> {
    let sender = inbox.sender();
    let op_config = operation_config.geogroup_params.to_owned();

    let mut flattened = hiearchy
        .iter()
        .flat_map(|h| h.leaves_cloned())
        .collect::<Vec<_>>();
    flattened.sort_by_key(|it| it.date);
    let ready_for_sorting = flattened
        .into_iter()
        .filter_map(|leaf| leaf.transform_for_sorting()) // TODO: Don't just filter out items without a position
        .collect();

    let geocoder = args.get_geocoder();

    std::thread::spawn(move || {
        sender
            .send(Message::SetProgress(Some(gui::Progress {
                action: gui::ProgressAction::Grouping,
                progress: None,
            })))
            .unwrap();

        let sorted_hiearchy = backend::algorithm::sort(ready_for_sorting, &op_config)
            .map_leafs(&|leaf| hiearchy::File::transform_after_sorting(leaf))
            .map_group_data(&|()| String::from("Group"));

        println!("Sorted!");

        sender
            .send(Message::SetProgress(Some(gui::Progress {
                action: gui::ProgressAction::Geocoding,
                progress: None,
            })))
            .unwrap();

        geocoder
            .prefetch_places(
                &sorted_hiearchy
                    .leaves()
                    .filter_map(|file| file.pos)
                    .collect(),
                |prog| {
                    sender
                        .send(Message::SetProgress(Some(gui::Progress {
                            action: gui::ProgressAction::Geocoding,
                            progress: Some(
                                (u16::MAX as f32 * prog.done as f32 / prog.total as f32) as u16,
                            ),
                        })))
                        .unwrap();
                },
            )
            .unwrap();

        sender
            .send(Message::SetProgress(Some(gui::Progress {
                action: gui::ProgressAction::Naming,
                progress: None,
            })))
            .unwrap();

        let named_hiearchy = geocoder
            .name_hiearchy(
                sorted_hiearchy
                    .map_leafs(&|file| {
                        (
                            file.pos
                                .expect("Missing point, TODO: Handle this correctly"),
                            file,
                        )
                    })
                    .map_group_data(&|_| ()),
            )
            .map_leafs(&|(name, file)| hiearchy::File { name, ..file });

        sender.send(Message::SetProgress(None)).unwrap();

        sender
            .send(Message::Sorted {
                new_hiearchy: match named_hiearchy {
                    backend::HiearchyItem::Group(g, _) => g,
                    backend::HiearchyItem::Item(it) => vec![backend::HiearchyItem::Item(it)],
                },
            })
            .unwrap();
    })
}
