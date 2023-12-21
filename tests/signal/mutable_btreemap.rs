use std::collections::BTreeMap;
use futures_signals::signal_map::{MapDiff, MutableBTreeMap, SignalMapExt};
use std::iter::FromIterator;
use futures_channel::mpsc::channel;
use futures_util::StreamExt;

#[tokio::test]
async fn test_filter_map() {
    let input = MutableBTreeMap::from([(1, "1".to_string()), (2, "2".to_string()), (3, "3".to_string())]);
    let output_signal = input.signal_map_cloned().filter(|v| v % 2 == 0);

    let output: MutableBTreeMap<i32, String> = MutableBTreeMap::new();
    let output_cloned = output.clone();

    let (mut proceed_tx, mut proceed_rx) = channel(100);

    tokio::spawn(
        output_signal.for_each(move |change| {
            let mut locked = output_cloned.lock_mut();

            match change {
                MapDiff::Replace { entries } => locked.replace_cloned(BTreeMap::from_iter(entries)),
                MapDiff::Remove { key } => { locked.remove(&key); }
                MapDiff::Insert { key, value } => { locked.insert_cloned(key, value); }
                MapDiff::Clear {} => locked.clear(),
                MapDiff::Update { key, value } => { locked.insert_cloned(key, value); }
            }

            println!("ijdoiqwjdqwd");
            proceed_tx.try_send(()).unwrap();

            async {}
        }));

    proceed_rx.next().await.unwrap();

    assert_eq!(output.lock_ref().len(), 1);
    assert_eq!(output.lock_ref().get_key_value(&2), Some((&2, &"2".to_string())));

    input.lock_mut().insert_cloned(42, "test".to_string());

    proceed_rx.next().await.unwrap();

    assert_eq!(output.lock_ref().len(), 2);
    assert_eq!(output.lock_ref().get_key_value(&42), Some((&42, &"test".to_string())));

    input.lock_mut().remove(&42);
    proceed_rx.next().await.unwrap();

    assert_eq!(output.lock_ref().len(), 1);
    assert_eq!(output.lock_ref().get_key_value(&42), None);
}