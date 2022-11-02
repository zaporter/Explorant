use std::{collections::HashMap, path::PathBuf};

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

pub_struct!(RecordedFramesResponse{
    frames: HashMap<String, Vec<u8>>,
});

pub_struct!(GeneralInfoRequest{});
pub_struct!(GeneralInfoResponse {
    binary_name: String,
    frame_time_map: FrameTimeMap,
    proc_maps: Vec<Map>,
});

pub_struct!(ScreenshotCaptures {
    
});
pub_struct!(TimeRange {
    start:TimeStamp,
    end:TimeStamp,
});

pub_struct!(TimeStamp{
    frame_time:usize,
    addr:Option<usize>,
    instance_of_addr:usize,
});

pub_struct!(SourceFileRequest{
    file_name:String,
});

pub_struct!(SourceFileResponse{
    data:String,
});

pub_struct!(GetFunctionData{
    
});
pub_struct!(FunctionTimeRangeRequest{
    range:TimeRange,
});
pub_struct!(FunctionTimeRangeResponse{
    addr_of_called_functions:Vec<usize>,
});

pub_struct!(FunctionInfoRequest{
    addr_of_function: usize,
});
pub_struct!(FunctionInfoResponse{
    function:Function,
});

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum LineItem {
    RawString(String),
    FunctionReference{address: usize},
    TypeReference{address:usize},
}
pub_struct!(Line{
    address: usize,
    items: Vec<LineItem>,
});
pub_struct!(Type {
    demangled_name: String,
});

pub_struct!(LineLocation{
    line_num:u32,
    column_num:u32,
});
pub_struct!(FileInfo {
    functions: Vec<Function>,
    lines: HashMap<usize,LineLocation>,
});

pub_struct!(Function{
    source_file: PathBuf,
    demangled_name: String,
    address: usize,
    size: usize,
    start_line: u32, 
    end_line: u32,
});


pub_struct!(FunctionExecutionHeatMapRequest{
    range:TimeRange,
    function_address: usize,
});
pub_struct!(FrameExecutionHeatMapResponse{
    map:FunctionExecutionHeatMap,
});

pub_struct!(FunctionExecutionHeatMap{
    addr_vs_times_executed: HashMap<usize,usize>,
});

pub_struct!( FrameTimeMap {
    frames: Vec<(i64, u128, String)>,
    times: HashMap<i64, u128>,
});

pub_struct!(ExecutionInfo{
    
});
