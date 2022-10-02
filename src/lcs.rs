use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, error::Error, mem, ops::DerefMut, rc::Rc};



// Ideally, one search through the list
// Paths:
// Map[addr->vec[addr]]
// O(n)
//
// Then create the tree from the Map
//
// addr -> [addrs]
// O(n*log(n))
//
//
//
#[derive(Default, Debug)]
pub struct BlockVocabulary {
    pub map: HashMap<usize, Rc<RefCell<InstructionSequence>>>,
    pub num_words: usize,
    next_word_id: usize,
}
impl BlockVocabulary {
    pub fn add_experience_to_vocabulary(&mut self, input: &Vec<usize>) {
        if !self.map.contains_key(&input[0]) {
            let first = Rc::new(RefCell::new(InstructionSequence::new(input[0])));
            first.borrow_mut().id = self.get_next_word_id();
            self.map.insert(input[0], first);
            self.num_words += 1;
        }

        for (last_addr, addr) in input.iter().tuple_windows() {
            // If the map does not contain the key then it is a new address
            if !self.map.contains_key(addr) {
                let last_sequence_len = self.map.get(last_addr).unwrap().borrow().exits().len();
                // If the current block does not have any exits, add it to the end of the current
                // block
                if last_sequence_len == 0 {
                    let last_sequence = self.map.get(last_addr).unwrap();
                    last_sequence.borrow_mut().insert(*addr);
                    self.map.insert(*addr, last_sequence.clone());
                // If the current block does have exits, then we need to create a new block
                // and have this block point to it. (the new block cannot be an existing exit of
                // the current block as it is new to the map)
                } else {
                    let last_sequence = self.map.get(last_addr).unwrap();
                    last_sequence.borrow_mut().insert_exit(*addr);
                    let new_child_block = Rc::new(RefCell::new(InstructionSequence::new(*addr)));
                    new_child_block.borrow_mut().id = self.get_next_word_id();
                    self.map.insert(*addr, new_child_block);
                    self.num_words += 1;
                }
            // Otherwise the address is already in the map
            } else {
                let last_sequence = self.map.get(last_addr).unwrap();
                // if the current block does not contain this as an exit
                // mark it as one
                if !last_sequence.borrow().exits().contains(addr) {
                    last_sequence.borrow_mut().insert_exit(*addr);
                }
                let referenced_sequence = self.map.get(addr).unwrap();
                // We need to split up referenced sequence because otherwise we are going
                // halfway inside of it.
                if referenced_sequence.borrow().base() != *addr {
                    let child = Rc::new(RefCell::new(
                        referenced_sequence.borrow_mut().split_at(*addr).unwrap(),
                    ));
                    child.borrow_mut().id = self.get_next_word_id();
                    self.num_words += 1;

                    // rewrite all the previous mappings
                    for entry in child.borrow().entries() {
                        self.map.insert(*entry, child.clone());
                    }
                }
            }
        }
    }
    fn get_next_word_id(&mut self) -> usize {
        self.next_word_id += 5;
        self.next_word_id
    }
    pub fn addrs_to_block_vocabulary(&self, addrs:&Vec<usize>)->Vec<usize>{
        let mut i = 0;
        let mut words = Vec::new();
        while i<addrs.len(){
            let sequence = self.map.get(&addrs[i]).unwrap().borrow();
            // dbg!(&addrs[i]);
            // dbg!(&sequence.addresses);
            // dbg!(&sequence.exits());
            i+=sequence.addresses.len();
            words.push(sequence.id());
        }
        words
    }
}
// This cannot be easily placed inside of an Interval Tree
#[derive(Debug)]
pub struct InstructionSequence {
    id: usize,
    addresses: Vec<usize>,
    exits: Vec<usize>,
}

impl InstructionSequence {
    pub fn new(base: usize) -> Self {
        Self {
            id: 0,
            addresses: vec![base],
            exits: Vec::new(),
        }
    }
    pub fn insert(&mut self, entry: usize) {
        self.addresses.push(entry);
    }
    pub fn base(&self) -> usize {
        self.addresses[0]
    }
    pub fn insert_exit(&mut self, exit: usize) {
        self.exits.push(exit);
    }
    pub fn entries(&self) -> &Vec<usize> {
        &self.addresses
    }
    pub fn exits(&self) -> &Vec<usize> {
        &self.exits
    }
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn split_at(&mut self, split_entry: usize) -> Result<Self, Box<dyn Error>> {
        let pos = self
            .addresses
            .iter()
            .position(|&k| k == split_entry)
            .ok_or("Cannot split at an entry that is not present")?;

        let mut child = InstructionSequence::new(split_entry);
        // give our child our exit addresses
        mem::swap(&mut child.exits, &mut self.exits);
        // ensure we are now poinint to our child
        self.exits.push(child.base());
        // give the child the right entries
        child.addresses = self.addresses.split_off(pos);
        Ok(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vocab_split_count() {
        let mut vocab = BlockVocabulary::default();
        vocab.add_experience_to_vocabulary(&vec![0, 1, 2, 1]);
        assert_eq!(vocab.num_words, 2);
        vocab.add_experience_to_vocabulary(&vec![0, 1]);
        assert_eq!(vocab.num_words, 2);
        vocab.add_experience_to_vocabulary(&vec![2, 3]);
        assert_eq!(vocab.num_words, 3);
        vocab.add_experience_to_vocabulary(&vec![4, 1]);
        assert_eq!(vocab.num_words, 4);
        // TODO: This could arguably simplify the graph
        // and allow for 3 and 4 to be merged into one word 
        // I don't think it hurts to leave it as it is though
        vocab.add_experience_to_vocabulary(&vec![3, 4]);
        assert_eq!(vocab.num_words, 4);

        dbg!(&vocab);
        let blk_vocab = vocab.addrs_to_block_vocabulary(&vec![0,1,2,3,4]);
        assert_eq!(blk_vocab,vec![5,10,15,20]);
    }
}
