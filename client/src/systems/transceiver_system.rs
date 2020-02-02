struct TransceiverSystem {}

impl<'a> System<'a> for TransceiverSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        println!("Hello there!");
    }
}
