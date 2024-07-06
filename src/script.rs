// use crossbeam::channel::{Sender, Receiver};
use async_channel::{Sender, Receiver};
use std::time::Duration;
use async_std::task;

pub enum Input {
    BrokeSeed,
    GoalReached,
    Abort,
}

pub enum Output {
    Message(String),
    End,
}

pub async fn level_vanilla(level_number: usize, input: Receiver<Input>, output: Sender<Output>) {
    loop {
        let x = input.recv().await.unwrap();
        if matches!(x, Input::GoalReached | Input::Abort) {
            break;
        }
    }
    task::sleep(Duration::from_millis(1000)).await;
    // We should add a wait here.
    output.send(Output::Message(format!("You passed level {}!!!", level_number))).await.unwrap();
    task::sleep(Duration::from_millis(1000)).await;
    output.send(Output::Message("Good job".into())).await.unwrap();
    task::sleep(Duration::from_millis(1000)).await;
    output.send(Output::End).await.unwrap();
}
