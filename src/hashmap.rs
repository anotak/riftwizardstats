use std::collections::HashMap;

pub trait HashMapExtensions<T : Eq,V> where T : Eq + std::hash::Hash {
    fn merge_add(self, other : Option<HashMap<T, V>>) -> Option<HashMap<T, V>>;
}

impl<T, V> HashMapExtensions<T, V> for Option<HashMap<T, V>>
    where T : Eq + std::hash::Hash + Clone,
        V : Copy + std::ops::Add<Output = V>
{
    fn merge_add(self, other : Option<HashMap<T, V>>) -> Option<HashMap<T, V>> 
    {
        match self {
            Some(mut a) => match other {
                Some(other) =>
                {
                    for (key, value) in other.iter() {
                        let sum = match a.get(key)
                            {
                                Some(old) => *old + *value,
                                None => *value,
                            };
                        
                        a.insert(key.clone(), sum);
                    }
                    
                    Some(a)
                },
                None => Some(a)
            },
            None => match other {
                Some(other) => {
                    let mut a = HashMap::new();
                    
                    for (key, value) in other.iter() {
                        a.insert(key.clone(), *value);
                    }
                    
                    Some(a)
                },
                None => None
            }
        }
    }
}

pub fn lazy_init<T, V>(map : Option<HashMap<T, V>>) -> HashMap<T, V>
{
    map.or_else(|| {Some(HashMap::new())}).unwrap()
}

