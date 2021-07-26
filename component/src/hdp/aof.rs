use tokio::{fs::File, io::BufWriter};

struct AofStatus {
    pub cur_id: u64,
    pub file: BufWriter<File>,
}

impl AofStatus {
    pub fn send(){
        
    }
}