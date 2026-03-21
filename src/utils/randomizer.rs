use rand::seq::SliceRandom;
use rand::thread_rng;

/// Randomize the order of a vector, returning a new vector with shuffled elements
pub fn randomize_order<T: Clone>(items: &[T]) -> Vec<T> {
    let mut rng = thread_rng();
    let mut shuffled = items.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled
}

/// Randomize and return indices mapping
/// Returns a vector of original indices in random order
pub fn randomize_indices(count: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..count).collect();
    let mut rng = thread_rng();
    indices.shuffle(&mut rng);
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_randomize_order() {
        let items = vec![1, 2, 3, 4, 5];
        let shuffled = randomize_order(&items);
        
        // Should have same length
        assert_eq!(shuffled.len(), items.len());
        
        // Should contain all original elements
        for item in &items {
            assert!(shuffled.contains(item));
        }
    }

    #[test]
    fn test_randomize_indices() {
        let count = 10;
        let indices = randomize_indices(count);
        
        // Should have correct length
        assert_eq!(indices.len(), count);
        
        // Should contain all indices 0..count
        for i in 0..count {
            assert!(indices.contains(&i));
        }
    }
}
