use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone)]
struct Entry<K>{
    val: K,
    ref_bit: bool
}

impl <K: Hash + Clone + Eq> Entry<K>{
    fn new(val: K)-> Self{
        Self{
            val,
            ref_bit: true
        }
    }
}


#[derive(Debug, Clone)]
pub struct PageCache<K>{
    entries: Vec<Entry<K>>,
    map: HashMap<K, usize>,
    hand: usize,
    capacity: usize,
}

impl<K: Hash + Clone + Eq +  Debug> PageCache<K>{
    pub fn new(capacity: usize) -> Self{
        assert!(capacity > 0);

        Self{
            map: HashMap::with_capacity(capacity),
            hand: 0,
            capacity,
            entries: Vec::with_capacity(capacity)
        }
    }

    fn touch(&mut self, key: &K) -> bool{
        if let Some(idx) = self.map.get(key){
            self.entries[*idx].ref_bit = true;
            return true
        }
        false
    }

    pub fn insert(&mut self, val: K) -> Option<K>{
        println!("inserting val: {val:?}");
        // check if the val exists and touch it
        let entry = Entry::new(val.clone());
        if self.touch(&val){
            println!("entries: {:?} hand: {}", self.entries, self.hand);
            return Some(val)
        }

        // if the map is not full, just add it
        if self.map.len() < self.capacity{
            let idx = self.entries.len();
            self.entries.push(entry.clone());

            self.map.insert(val.clone(), idx);
            println!("entries: {:?} hand: {}", self.entries, self.hand);
            return Some(val);
        }

        // start the clock


        loop{
            let old_entry = &self.entries[self.hand];
            if old_entry.ref_bit{
                // give it a second chance
                self.entries[self.hand].ref_bit = false;
                self.hand = (self.hand + 1) % self.capacity;
                continue;
            }
            else{
                // we found a culprit!
                self.map.remove(&old_entry.val);
                self.map.insert(val.clone(), self.hand);
                self.entries[self.hand] = entry.clone();


                // move hand one more
                self.hand = (self.hand + 1) % self.capacity;
                break;
            }
        }
        println!("entries: {:?} hand: {}", self.entries, self.hand);
        Some(val)
    }
}


fn main() {
    let mut pc: PageCache<u32> = PageCache::new(5);
    pc.insert(1);
    pc.insert(2);
    pc.insert(3);
    pc.insert(4);
    pc.insert(5);
    pc.insert(6);
    pc.insert(2);
    pc.insert(1);
    pc.insert(3);
    pc.insert(4);
    pc.insert(5);
    pc.insert(2);
    pc.insert(6);
}