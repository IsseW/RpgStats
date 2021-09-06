#[derive(Default, PartialOrd, PartialEq)]
pub struct OrderedFloat(pub f32);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
