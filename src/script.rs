// use crossbeam::channel::{Sender, Receiver};
use async_channel::{Sender, Receiver};

pub enum Input {
    BrokeSeed,
    GoalReached,
}

pub enum Output {
    Message(String),
    End,
}

pub async fn level_vanilla(level_number: usize, input: Receiver<Input>, output: Sender<Output>) {
    loop {
        let x = input.recv().await.unwrap();
        if matches!(x, Input::GoalReached) {
            break;
        }
    }
    // We should add a wait here.
    output.send(Output::Message(format!("You passed level {}!!!", level_number))).await.unwrap();
    output.send(Output::End).await.unwrap();
}
