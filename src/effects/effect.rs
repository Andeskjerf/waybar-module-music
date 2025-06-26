pub trait Effect: Send + Sync {
    fn apply(&mut self, text: String) -> String;
    fn set_text(&mut self, text: String);
    fn update_active(&mut self);
    fn is_active(&self) -> bool;
}
