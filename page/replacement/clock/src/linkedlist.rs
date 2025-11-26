use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Eq, PartialOrd, PartialEq)]
pub struct Page {
    pub val: u32,
    pub bit: u8,
    pub next: Option<Rc<RefCell<Page>>>
}


impl Page{
    fn new(val: u32, next: Option<Rc<RefCell<Page>>>)-> Self{
        Self{
            val,
            bit: 0b1,
            next
        }
    }

}


#[derive(Debug, Clone)]
pub struct PageCache {
    head: Rc<RefCell<Page>>,
    tail: Rc<RefCell<Page>>,
    capacity: usize,
    count: usize,
    pub(crate) current: Rc<RefCell<Page>>,
    all_set: u8
}


impl PageCache {
    pub fn new(capacity: usize) -> Self{
        assert!(capacity > 0);
        let dummy = Rc::new(RefCell::new(Page::new(0, None)));
        let this = Self{
            head: dummy.clone(),
            tail: dummy.clone(),
            capacity,
            count: 0,
            current: dummy.clone(),
            all_set: 0b0
        };
        this
    }


    pub fn insert(&mut self, val: u32) -> &mut Self {
        let page = Rc::new(RefCell::new(Page::new(val, None)));

        // try to search for the page
        if self.count == 0{
            // we are the first page
            let node = Rc::new(RefCell::new(Page::new(val, None)));
            node.borrow_mut().next = Some(node.clone()); // circular
            self.head = node.clone();
            self.tail = node.clone();
            self.current = node.clone();
            self.count += 1;
            return self;
        }

        let mut curr = self.head.clone();

        loop {
            if curr.borrow().val == val{
                // just set the bit and return
                curr.borrow_mut().bit = 0b1;
                return self
            }
            let next = curr.borrow().next.as_ref().cloned().unwrap();
            curr = next;

            if Rc::ptr_eq(&curr, &self.head){
                // we've gotten to the head. Does not exist
                break
            }
        }


        if self.count < self.capacity {
            // add to the list(the tail)
            let node = Rc::new(RefCell::new(Page::new(val, Some(self.head.clone()))));
            self.tail.borrow_mut().next = Some(node.clone());
            self.tail = node.clone();
            self.count += 1;
        }
        else {
            self.clock_replace(val);
        }
        self
    }


    fn clock_replace(&mut self, val: u32)-> &mut Self{
        // short circuit reset all nodes
        if self.all_set == 0b1{
            self.reset_nodes();
        }

        // start the clock
        let mut curr_selected = self.current.clone();
        loop {
            if curr_selected.borrow().bit == 0b1{
                // give it a second chance and move on
                curr_selected.borrow_mut().bit = 0b0;
                let next = curr_selected.borrow().next.as_ref().cloned().unwrap();
                curr_selected = next;
                continue
            }

            // we found a candidate to replace
            curr_selected.borrow_mut().val = val;
            curr_selected.borrow_mut().bit = 0b1;
            // shift the current node one more
            let next = curr_selected.borrow().next.as_ref().cloned().unwrap();
            self.current = next;
            break
        }

        self
    }

    fn reset_nodes(&mut self){
        let mut curr = self.head.clone();

        loop {
            curr.borrow_mut().bit = 0;
            let next = curr.borrow().next.as_ref().cloned().unwrap();

            if Rc::ptr_eq(&curr, &self.tail) {
                break;
            }

            curr = next;
        }
    }
    
    /// save printing the linked-list with boundary-checking to prevent 
    /// stack overflow
    pub fn debug_values(&self) -> Vec<(u32, u8)> {
        let mut out = Vec::new();

        if self.count == 0 {
            return out;
        }
        
        let mut node = self.head.clone();

        for _ in 0..self.count {
            out.push((node.borrow().val, node.borrow().bit));
            let next_opt = node.borrow().next.as_ref().cloned();
            match next_opt {
                Some(next_rc) => node = next_rc, 
                None => break,
            }
        }

        out
    }
}

