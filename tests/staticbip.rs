use staticbip::StaticBip;

#[test]
fn read_empty() {
    let mut buffer = StaticBip::<u8, 3>::default();
    assert!(buffer.is_empty());
    assert!(buffer.read().is_empty());
}

#[test]
fn read_uncommitted() {
    let mut buffer = StaticBip::<u8, 3>::default();
    assert_eq!(buffer.reserve(2).len(), 2);
    assert!(buffer.read().is_empty());
}

#[test]
fn reserve_gt_overall_len() {
    let mut buffer = StaticBip::<u8, 3>::default();
    assert_eq!(buffer.reserved(), 0);
    assert_eq!(buffer.reserve(4).len(), 3);
    assert_eq!(buffer.reserved(), 3);
}

#[test]
fn commit_and_fetch() {
    let mut buffer = StaticBip::<u8, 4>::default();
    buffer.reserve(3).copy_from_slice(&[1, 2, 3]);

    assert_eq!(buffer.committed(), 0);
    buffer.commit(3);

    assert_eq!(buffer.committed(), 3);
    assert_eq!(buffer.reserved(), 0);

    assert_eq!(buffer.read(), &[1, 2, 3]);
}

#[test]
fn reserve_full() {
    let mut buffer = StaticBip::<u8, 4>::default();
    assert_eq!(buffer.reserve(4).len(), 4);
    buffer.commit(4);
    assert!(buffer.reserve(1).is_empty());
}

#[test]
fn decommit() {
    let mut buffer = StaticBip::<u8, 4>::default();
    buffer.reserve(4).copy_from_slice(&[1, 2, 3, 4]);

    buffer.commit(4);
    assert_eq!(buffer.read(), &[1, 2, 3, 4]);

    buffer.decommit(2);
    assert_eq!(buffer.read(), &[3, 4]);

    buffer.decommit(1);
    assert_eq!(buffer.read(), &[4]);
}

#[test]
fn reserve_after_full_cycle() {
    let mut buffer = StaticBip::<u8, 4>::default();
    buffer.reserve(4).copy_from_slice(&[1, 2, 3, 4]);

    buffer.commit(4);
    buffer.decommit(2);

    buffer.reserve(4).copy_from_slice(&[5, 6]);

    buffer.commit(2);
    assert_eq!(buffer.read(), &[3, 4]);

    buffer.decommit(2);
    assert_eq!(buffer.read(), &[5, 6]);
}

#[test]
fn clear() {
    let mut buffer = StaticBip::<u8, 4>::default();
    buffer.reserve(4).copy_from_slice(&[1, 2, 3, 4]);

    assert_eq!(buffer.reserved(), 4);

    buffer.commit(4);
    assert_eq!(buffer.reserved(), 0);

    buffer.clear();
    assert_eq!(buffer.committed(), 0);
}

#[test]
fn pop() {
    let mut buffer = StaticBip::<usize, 4>::default();

    buffer.reserve(4).copy_from_slice(&[1234, 0, 1, 2]);
    buffer.commit(4);

    buffer.decommit(1);

    buffer.reserve(1).copy_from_slice(&[3]);
    buffer.commit(1);

    for i in 0..4 {
        assert_eq!(i, buffer.pop().copied().unwrap());
    }
}
