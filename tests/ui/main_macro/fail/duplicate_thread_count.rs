use chronographer::main;

#[main(thread_count = 4, thread_count = 8)]
async fn main(sched: MyScheduler) {}