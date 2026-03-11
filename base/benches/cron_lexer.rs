use chronographer_base::task::schedule::cron_lexer::tokenize_fields;

fn main() {
    divan::main();
}

#[divan::bench]
fn tokenize_simple_wildcard() {
    divan::black_box(tokenize_fields("0 0 * * * *").unwrap());
}

#[divan::bench]
fn tokenize_numeric_all_fields() {
    divan::black_box(tokenize_fields("30 15 12 1 6 3").unwrap());
}

#[divan::bench]
fn tokenize_step_expression() {
    divan::black_box(tokenize_fields("*/15 */30 */6 * * *").unwrap());
}

#[divan::bench]
fn tokenize_range_expression() {
    divan::black_box(tokenize_fields("0-30 0-59 9-17 1-15 * 1-5").unwrap());
}

#[divan::bench]
fn tokenize_list_expression() {
    divan::black_box(tokenize_fields("0,15,30,45 0 12 1,15 * *").unwrap());
}

#[divan::bench]
fn tokenize_last_modifier() {
    divan::black_box(tokenize_fields("0 0 0 L * *").unwrap());
}

#[divan::bench]
fn tokenize_mixed_operators() {
    divan::black_box(tokenize_fields("*/5 0-30 12,18 */2 * ?").unwrap());
}

#[divan::bench]
fn tokenize_five_fields() {
    divan::black_box(tokenize_fields("*/10 * * * *").unwrap());
}
