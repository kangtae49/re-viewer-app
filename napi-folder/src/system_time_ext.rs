pub trait SystemTimeExt {
    fn to_sec(&self) -> u64;
    // fn to_ms(&self) -> u128;
}

impl SystemTimeExt for std::time::SystemTime {
    fn to_sec(&self) -> u64 {
        match self.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    // fn to_ms(&self) -> u128 {
    //     self.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
    // }   
}