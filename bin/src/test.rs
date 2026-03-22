use std::str::FromStr;
use chronographer::every;
use chronographer::prelude::TaskScheduleCron;

#[tokio::main]
async fn main() {
    let abc = every!(3d 1s);

    let val = TaskScheduleCron::from_str("*/3 LW L-3 3L * *")
        .unwrap();
}
