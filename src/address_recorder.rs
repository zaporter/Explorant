use crate::shared_structs::*;
// I use a dual layered system to record entries in this table
// The first is a map of frametime -> FtBin
//
// The FtBin stores the addresses and then a map of TimeStamp->index
pub struct AddressRecorder {
    records: Vec<FtBin>,
    write_head: usize,
    is_writing_ftbin: bool,
}
// TODO
// I hate PhantomData and refuse to use it for this
// This object is not threadsafe.
//
//impl !Sync for AddressRecorder {}

#[derive(Default, Clone)]
struct FtBin {
    pub addresses: Vec<usize>,
    pub indexes: Vec<(TimeStamp, usize)>,
}

impl AddressRecorder {
    pub fn new(max_frame_time: usize) -> Self {
        Self {
            write_head: 0,
            is_writing_ftbin: false,
            // plus two because we start at 1 and are inclusive
            // at the end. I could do this correctly
            // but that adds more mental overhead than the
            // few extra bytes are worth
            records: vec![FtBin::default(); max_frame_time + 2],
        }
    }
    pub fn reset_ft_for_writing(&mut self, frame_time: usize) {
        debug_assert!(!self.is_writing_ftbin);
        self.records[frame_time].indexes.clear();
        self.records[frame_time].addresses.clear();

        self.write_head = frame_time;
        self.records[frame_time]
            .indexes
            .push((TimeStamp::new_at_ft(frame_time), 0));
        self.is_writing_ftbin = true;
    }
    pub fn insert_address(&mut self, address: usize) {
        debug_assert!(self.is_writing_ftbin);
        self.records[self.write_head].addresses.push(address);
    }
    pub fn insert_timestamp(&mut self, stamp: TimeStamp) {
        debug_assert!(self.is_writing_ftbin);
        debug_assert!(stamp.frame_time == self.write_head);
        let current_loc = self.records[self.write_head].addresses.len() - 1;
        self.records[self.write_head]
            .indexes
            .push((stamp, current_loc));
    }
    pub fn finished_writing_ft(&mut self) {
        debug_assert!(self.is_writing_ftbin);
        self.is_writing_ftbin = false;
        // placeholder
    }
    // return a vec of slices to
    // avoid copying large amounts of
    // memory
    pub fn get_addresses_in<'a>(&'a self, range: &TimeRange) -> anyhow::Result<AddrIter<'a>> {
        // main cases:
        // invalid range
        //
        let mut to_ret = AddrIter::new();
        let start = &range.start;
        let end = &range.end;
        // go to start_bin
        let start_bin = &self.records[start.frame_time];
        let start_index_entry = start_bin
            .indexes
            .iter()
            .find(|stamp| stamp.0 == *start)
            .ok_or(anyhow::Error::msg("Invalid start of time range"))?;
        let end_bin = &self.records[end.frame_time];
        let end_index_entry = end_bin
            .indexes
            .iter()
            .find(|stamp| stamp.0 == *end)
            .ok_or(anyhow::Error::msg("Invalid end of time range"))?;
        // assert that the start happens before the end
        if start.frame_time == end.frame_time {
            if start_index_entry.1 > end_index_entry.1 {
                Err(anyhow::Error::msg(
                    "The end of the range came before the beginning! (entry)",
                ))?;
            }
        } else if start.frame_time > end.frame_time {
            Err(anyhow::Error::msg(
                "The end of the range came before the beginning! (frametime)",
            ))?;
        }
        // rust needs do..while loops. Not having them is stupid.
        let mut current_frame_entry = start_index_entry;
        loop {
            let current_bin = &self.records[current_frame_entry.0.frame_time];

            if current_frame_entry.0.frame_time == end_index_entry.0.frame_time {
                to_ret
                    .chunks
                    .push(&current_bin.addresses[(current_frame_entry.1)..(end_index_entry.1)]);
                break;
            } else {
                to_ret
                    .chunks
                    .push(&current_bin.addresses[(current_frame_entry.1)..]);
            }
            let mut next_entered_frame_time = current_frame_entry.0.frame_time + 1;
            while self.records[next_entered_frame_time].addresses.len() == 0 {
                next_entered_frame_time += 1;
            }
            current_frame_entry =
                self.records[next_entered_frame_time]
                    .indexes
                    .get(0)
                    .ok_or(anyhow::Error::msg(
                        "Tried to read from uninitlialized. This should be impossible",
                    ))?;
        }
        Ok(to_ret)
    }
}

pub struct AddrIter<'a> {
    pub chunks: Vec<&'a [usize]>,
    index: usize,
    index_index: usize,
}
impl<'a> AddrIter<'a> {
    fn new() -> Self {
        Self {
            chunks: Vec::new(),
            index: 0,
            index_index: 0,
        }
    }
}
impl<'a> Iterator for AddrIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.chunks.get(self.index) {
            Some(chunk) => {
                self.index_index += 1;
                match chunk.get(self.index_index - 1) {
                    // If the child has an entry at the subindex,
                    // return that
                    Some(entry) => Some(*entry),
                    // otherwise go to the next chunk and run again
                    None => {
                        self.index += 1;
                        self.index_index = 0;
                        self.next()
                    }
                }
            }
            None => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_insert() {
        let mut ar = AddressRecorder::new(2);
        ar.reset_ft_for_writing(1);
        ar.insert_address(0);
        ar.finished_writing_ft();
        ar.reset_ft_for_writing(2);
        ar.insert_address(1);
        ar.finished_writing_ft();
        let start = TimeStamp::new_at_ft(1);
        let end = TimeStamp::new_at_ft(2);
        let result: Vec<usize> = ar
            .get_addresses_in(&TimeRange { start, end })
            .unwrap()
            .collect();
        dbg!(&result);
        assert_eq!(result, vec![0]);
    }
    #[test]
    fn middle_end() {
        let mut ar = AddressRecorder::new(2);
        let start = TimeStamp::new_at_ft(1);
        let end = TimeStamp {
            frame_time: 2,
            addr: Some(5),
            instance_of_addr: Some(5),
        };
        ar.reset_ft_for_writing(1);
        ar.insert_address(0);
        ar.finished_writing_ft();
        ar.reset_ft_for_writing(2);
        ar.insert_address(1);
        ar.insert_address(3);
        ar.insert_address(4);
        ar.insert_timestamp(end.clone());
        ar.insert_address(5);
        ar.finished_writing_ft();
        let result: Vec<usize> = ar
            .get_addresses_in(&TimeRange { start, end })
            .unwrap()
            .collect();
        dbg!(&result);
        assert_eq!(result, vec![0, 1, 3]);
    }
    #[test]
    fn same_start_and_end() {
        let mut ar = AddressRecorder::new(2);
        let start = TimeStamp {
            frame_time: 1,
            addr: Some(1),
            instance_of_addr: Some(5),
        };
        let end = TimeStamp {
            frame_time: 1,
            addr: Some(5),
            instance_of_addr: Some(5),
        };
        ar.reset_ft_for_writing(1);
        ar.insert_address(0);
        ar.insert_address(1);
        ar.insert_timestamp(start.clone());
        ar.insert_address(3);
        ar.insert_address(4);
        ar.insert_timestamp(end.clone());
        ar.insert_address(5);
        ar.finished_writing_ft();
        let result: Vec<usize> = ar
            .get_addresses_in(&TimeRange { start, end })
            .unwrap()
            .collect();
        dbg!(&result);
        assert_eq!(result, vec![1, 3]);
    }
    #[test]
    fn missing_middle() {
        let mut ar = AddressRecorder::new(5);
        let start = TimeStamp {
            frame_time: 1,
            addr: Some(1),
            instance_of_addr: Some(5),
        };
        let end = TimeStamp {
            frame_time: 5,
            addr: Some(5),
            instance_of_addr: Some(5),
        };
        let second_end = TimeStamp {
            frame_time: 5,
            addr: Some(6),
            instance_of_addr: Some(5),
        };
        ar.reset_ft_for_writing(1);
        ar.insert_address(0);
        ar.insert_address(1);
        ar.insert_timestamp(start.clone());
        ar.insert_address(3);
        ar.finished_writing_ft();
        ar.reset_ft_for_writing(3);
        ar.insert_address(1000);
        ar.finished_writing_ft();
        ar.reset_ft_for_writing(5);
        ar.insert_address(4);
        ar.insert_timestamp(end.clone());
        ar.insert_address(5);
        ar.insert_timestamp(second_end.clone());
        ar.finished_writing_ft();
        let result: Vec<usize> = ar
            .get_addresses_in(&TimeRange {
                start: start.clone(),
                end,
            })
            .unwrap()
            .collect();
        dbg!(&result);
        assert_eq!(result, vec![1, 3, 1000]);
        let result: Vec<usize> = ar
            .get_addresses_in(&TimeRange {
                start,
                end: second_end,
            })
            .unwrap()
            .collect();
        dbg!(&result);
        assert_eq!(result, vec![1, 3, 1000, 4]);
    }
}
