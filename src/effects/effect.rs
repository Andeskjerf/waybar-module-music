pub trait Effect {
    fn apply(&mut self, text: String) -> String;
}
