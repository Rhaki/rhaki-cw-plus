use std::collections::HashMap;

pub fn vec_to_i_hashmap<T>(vec: Vec<T>) -> HashMap<usize, T> {
    let mut map: HashMap<usize, T> = HashMap::new();

    for (i, v) in vec.into_iter().enumerate() {
        map.insert(i, v);
    }

    map
}
