use std::{collections::HashMap, fs::File, path::Path};

use nix::unistd::{fork, ForkResult};
use tracing::{error, info};

use super::slot::Slot;
use crate::utils::ParseSerdeType;

pub fn save_slots(slots: &HashMap<u16, Slot>) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            info!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
        }
        Ok(ForkResult::Child) => {
            for (id, slot) in slots {
                save_slot(*id, slot);
            }
        }
        Err(e) => error!("Fork failed: {}", e),
    }
}

fn save_slot(id: u16, slot: &Slot) {
    let file_name = format!("slot_{}", id);
    let path = Path::new(&file_name);
    let display = path.display();
    let file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    bincode::serialize_into(file, &slot.parse_serde_type()).unwrap();
}
