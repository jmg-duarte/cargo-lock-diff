use std::{collections::HashSet, fmt::Debug};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difference<T> {
    Empty,
    Equal(T),
    Removed(T),
    Modified { old: T, new: T },
    Added(T),
}

impl<T> Difference<T>
where
    T: PartialEq,
{
    pub fn is_equal(&self) -> bool {
        matches!(self, Difference::Equal(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Difference::Empty)
    }

    pub fn diff(a: T, b: T) -> Difference<T> {
        if a == b {
            Difference::Equal(a)
        } else {
            Difference::Modified { old: a, new: b }
        }
    }

    pub fn diff_opt(a: Option<T>, b: Option<T>) -> Difference<T> {
        match (a, b) {
            (None, None) => Difference::Empty,
            (None, Some(b)) => Difference::Added(b),
            (Some(a), None) => Difference::Removed(a),
            (Some(a), Some(b)) => {
                if a == b {
                    Difference::Equal(a)
                } else {
                    Difference::Modified { old: a, new: b }
                }
            }
        }
    }
}

impl<T> Difference<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn unstable_diff_vec(a: Vec<T>, b: Vec<T>) -> Vec<Difference<T>> {
        let a: HashSet<T> = HashSet::from_iter(a.into_iter());
        let b: HashSet<T> = HashSet::from_iter(b.into_iter());

        let mut diff = Vec::with_capacity(a.len().max(b.len()));
        for dependency in a.intersection(&b) {
            diff.push(Difference::Equal(dependency.clone()));
        }

        for dependency in a.difference(&b) {
            diff.push(Difference::Removed(dependency.clone()))
        }

        for dependency in b.difference(&a) {
            diff.push(Difference::Added(dependency.clone()))
        }
        diff
    }
}

impl<T> Difference<T>
where
    T: Eq + Ord + std::hash::Hash + Clone,
{
    pub fn diff_vec(a: Vec<T>, b: Vec<T>) -> Vec<Difference<T>> {
        let mut diff = Self::unstable_diff_vec(a, b);
        diff.sort();
        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        assert_eq!(Difference::diff(1, 1), Difference::Equal(1));
        assert_eq!(
            Difference::diff(1, 2),
            Difference::Modified { old: 1, new: 2 }
        );
    }

    #[test]
    fn test_diff_opt() {
        assert_eq!(Difference::<u8>::diff_opt(None, None), Difference::Empty);
        assert_eq!(Difference::diff_opt(Some(1), None), Difference::Removed(1));
        assert_eq!(Difference::diff_opt(None, Some(1)), Difference::Added(1));
        assert_eq!(Difference::diff_opt(Some(1), Some(1)), Difference::Equal(1));
        assert_eq!(
            Difference::diff_opt(Some(1), Some(2)),
            Difference::Modified { old: 1, new: 2 }
        );
    }

    #[test]
    fn test_diff_vec() {
        let a = vec!["a", "b", "c"];
        let b = vec!["b", "c", "d"];
        let expected = vec![
            Difference::Removed("a"),
            Difference::Equal("b"),
            Difference::Equal("c"),
            Difference::Added("d"),
        ];
        assert_eq!(Difference::diff_vec(a, b), expected);
    }
}
