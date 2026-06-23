use chronographer::main;


#[main(thread_count=6)]
async fn main(sched: MyScheduler) {}