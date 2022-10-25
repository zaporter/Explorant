use procmaps::Map;
use serde::{Serialize,Deserialize};
// from https://stackoverflow.com/questions/53866508/how-to-make-a-public-struct-where-all-fields-are-public-without-repeating-pub
//
// It really would be great if rust added a way to indicate that all elements of a struct are
// pub...
//
macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty),* $(,)?}) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // ewww
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(PingRequest {
    id:usize
});
pub_struct!(PingResponse {
    id:usize 
});
pub_struct!(EmptyRequest{});
pub_struct!(InstructionPointerResponse{
    instruction_pointer:usize,
});

pub_struct!(GeneralInfoRequest{});
pub_struct!(GeneralInfoResponse {
    binary_name: String,
    start_time_millis:usize,
    end_time_millis:usize,
    // map of frame_time->time_millis
    frame_time_map: Vec<(usize,usize)>,
    proc_maps: Vec<Map>,
});

pub_struct!(ScreenshotCaptures {
    
});
pub_struct!(FrameTimeRange {
    frame_time_start:usize,
    frame_time_end:usize,
});

// pub_struct!(RequestAllP)

pub_struct!(RequestAllFunctionsRun {


});
