use mutations::executor::{
    execute,
    Conf,
    MaxIterationCount
};

fn main() {
    execute(&Conf {
        file_name: "resources/deterministic_game_of_life.txt",
        with_display: false,
        iteration_delay: 0,
        max_iteration_count: MaxIterationCount::Finite(5000),
    });
}
