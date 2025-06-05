pub trait Effect {
    fn apply(&mut self, text: &str) -> String;
}
