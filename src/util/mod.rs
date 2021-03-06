pub use self::collision_world::CollisionWorld;

pub mod collision_world;

pub trait Transition {
    fn create_state(self) -> Option<Box<dyn State<Self>>>;
}

pub trait State<T> {
    fn run(self: Box<Self>) -> T;
}

pub fn run_state_machine<T: Transition>(initial: T) {
    let mut transition = initial;

    while let Some(s) = transition.create_state() {
        transition = s.run();
    }
}

#[cfg(test)]
mod test {
    use super::{run_state_machine, State, Transition};

    enum TestTransition {
        First,
        Last,
        Over,
    }

    impl Transition for TestTransition {
        fn create_state(self) -> Option<Box<dyn State<TestTransition>>> {
            match self {
                TestTransition::First => Some(Box::new(FirstState)),
                TestTransition::Last => Some(Box::new(LastState)),
                TestTransition::Over => None,
            }
        }
    }

    struct FirstState;
    struct LastState;

    impl State<TestTransition> for FirstState {
        fn run(self: Box<Self>) -> TestTransition {
            TestTransition::Last
        }
    }

    impl State<TestTransition> for LastState {
        fn run(self: Box<Self>) -> TestTransition {
            TestTransition::Over
        }
    }

    #[test]
    fn simple_state_machine() {
        run_state_machine(TestTransition::First)
    }
}
