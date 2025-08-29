use bytebuffer::ByteBuffer;

pub trait StateTile {
    fn save(&self, saver: &mut ByteBuffer);
    fn load_into(&mut self, loader: &mut ByteBuffer) -> Result<(), String>;
    fn save_for_client(&self, saver: &mut ByteBuffer);
    fn load_from_client(&mut self, loader: &mut ByteBuffer) -> Result<(), String>;
    fn box_clone(&self) -> Box<dyn StateTile>;
}