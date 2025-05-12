pub struct ServerTickEvent;

pub struct ServerCommandEvent {
    pub has_been_cancelled: bool,
    pub command: String
}