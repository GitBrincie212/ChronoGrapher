use chronographer::prelude::TaskScheduleCron;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let val = TaskScheduleCron::from_str("*/3 LW L-3 3L * *").unwrap();
}
