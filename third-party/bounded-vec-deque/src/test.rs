#![cfg(test)]

use ::BoundedVecDeque;

#[test]
fn max_size_zero() {
    let mut vector = BoundedVecDeque::new(0);

    assert_eq!(vector.push_back(0), Some(0));
    assert_eq!(vector.push_front(1), Some(1));
    assert_eq!(vector.insert_spill_back(0, 2), Some(2));
    assert_eq!(vector.insert_spill_front(0, 3), Some(3));
    let mut other_vector = BoundedVecDeque::from_iter(4..6, 2);
    assert!(vector.append(&mut other_vector).eq(vec![4, 5]));
    vector.extend(6..8);
    assert_eq!(vector.len(), 0);
}
