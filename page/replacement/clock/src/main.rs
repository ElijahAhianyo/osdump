mod linkedlist;
mod map;


fn linkedlist_main(){
    let mut pc = linkedlist::PageCache::new(5);
    pc.insert(1).insert(2).insert(3).insert(4).insert(5);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(6);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(2);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(1);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(3);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(4);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(5);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(2);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
    pc.insert(6);
    println!("pc: {:?} | curr_clock_val: {:?}", pc.debug_values(), pc.current.borrow().val);
}

fn map_main(){
    let mut pc: map::PageCache<u32> = map::PageCache::new(5);
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

fn main() {
    println!("Hello, world!");
}
