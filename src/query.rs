use std::rc::Rc;

pub enum Fidelity {
    // Dynamic code segment instrumentation with singlesteps
    DynamicEveryInstruction, // Very slow
    DynamicHighFidelity, // 2000-4000x original time
    // Use static code segment instrumentation
    StaticHighFidelity, // 5-10x original time
    StaticLowFidelity, // 1-2x original time
}
impl Fidelity {
    pub fn get_higher<'a> (&'a self, other:&'a Fidelity) -> &Fidelity{
        if self.to_int() > other.to_int() {
            return self;
        }else {
            return other;
        }
    }
    fn to_int(&self) -> usize{
        match self {
            Self::StaticLowFidelity=>0,
            Self::StaticHighFidelity=>1,
            Self::DynamicHighFidelity=> 9,
            Self::DynamicEveryInstruction=> 10,
        }
    }
}

pub enum QueryNode{
    TimeRange(usize,usize),
    FidelityFilter(Rc<QueryNode>, Fidelity),
    // MemoryRange(usize,usize),
    // Union(Rc<Selection>, Rc<Selection>),
    // Intersection(Rc<Selection>, Rc<Selection>),
    // Exclude(Rc<Selection>, Rc<Selection>),
    RunCountFilter(Option<usize>,Option<usize>),
    // SectionFilter(Vec<String>), // memory mapped sections to monitor
    // ThreadFilter(Vec<String>),
    Query(Box<dyn Query>),
}



// Jump Query
//
pub trait Query {
    fn run_on(child: Rc<QueryNode>) where Self:Sized;
}
