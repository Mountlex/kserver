pub trait CostMetric<T> {
    fn diff(&self, other: &Self) -> T;
}
