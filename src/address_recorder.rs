use crate::shared_structs::*;
// I use a dual layered system to record entries in this table
// The first is a map of frametime -> FtRecord
//
// The FtRecord stores the addresses and then a map of TimeStamp->index
struct AddressRecorder {
    records : Vec<FtRecord>,
    write_head : usize,
}

#[derive(Default,Clone)]
struct FtRecord {
    pub addresses : Vec<usize>,
    pub indexes : Vec<(TimeStamp, usize)>,
}

impl AddressRecorder {
    pub fn new(max_frame_time:usize) -> Self{ 
        Self {
            write_head : 0,
            records : vec![FtRecord::default();max_frame_time+2]    
        }
    }
    pub fn reset_ftrecord_for_writing(&mut self, frame_time: usize) {
        self.records[frame_time].indexes.clear();
        self.records[frame_time].addresses.clear();

        self.write_head = frame_time;
        self.records[frame_time].indexes.push((TimeStamp::new_at_ft(frame_time),0))
    }
    pub fn insert_address(&mut self, address: usize){
        self.records[self.write_head].addresses.push(address);
    }
    pub fn insert_timestamp(&mut self, stamp: TimeStamp){
        debug_assert!(stamp.frame_time == self.write_head);
        let current_loc = self.records[self.write_head].addresses.len()-1;
        self.records[self.write_head].indexes.push((stamp, current_loc));
    }
    pub fn finished_writing_ftrecord(&mut self){
        // placeholder
    }
    // return a vec of slices to 
    // avoid copying large amounts of
    // memory
    pub fn get_addresses_in(&self, range : &TimeRange) -> anyhow::Result<Vec<&[usize]>>{
        let to_ret = Vec::new();
        let start = &range.start;
        let end = &range.end;
        
        Ok(to_ret)
    }
}
