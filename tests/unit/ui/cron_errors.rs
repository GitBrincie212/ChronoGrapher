use chronographer::prelude::*;

fn main() {
    cron!(); // Expected one or more tokens, found an abrupt end

    cron!(MONDAY * * * * *); // Unknown character

    cron!(* * * * * * *); // Unexpected token sequence found

    cron!(60 * * * * *); // Value 60 is out of range for seconds field (expected 0-59)
    cron!(* 60 * * * *); // Value 60 is out of range for minutes field (expected 0-59)
    cron!(* * 24 * * *); // Value 24 is out of range for hours field (expected 0-23)
    cron!(* * * 32 * *); // Value 32 is out of range for day_of_month field (expected 1-31)
    cron!(* * * * 13 *); // Value 13 is out of range for month field (expected 1-12)
    cron!(* * * ? * 8); // Value 8 is out of range for day_of_week field (expected 1-7)

    cron!(*/0 * * * * *); // Step value 0 must be greater than 0
    cron!(5-1 * * * * *); // Invalid range 5-1 for seconds field (expected 0-59)

    cron!(? * * * * *); // Field 'seconds' cannot be unspecified in this context
    cron!(* * * * ? *); // Field 'month' cannot be unspecified in this context

    cron!(60/2 * * * * *); // Value 60 is out of range for seconds field (expected 0-59)
    cron!(100-200/2 * * * * *); // Invalid range 100-200 for seconds field (expected 0-59)

    cron!(* * * * L *); // L (last) operator is only valid for day_of_month and day_of_week fields
    cron!(W * * * * *); // Expected atom operator but got something else
    cron!(1#2 * * * * *); // # (nth weekday) operator is only valid for day_of_week field
    cron!(* * * ? * 1#6); // Nth weekday 6 is out of range (expected 1-5)
    cron!(* * * ? * 0#1); // Value 0 is out of range for day_of_week field (expected 1-7)

    cron!(0 0 0 L-0 * *); // Value 0 is out of range for day_of_month field (expected 1-30)
    cron!(0 0 0 ? * 8L); // Value 8 is out of range for day_of_week field (expected 1-7)
    cron!(0 0 0 L/2 * *); // Expected number but got something else

    cron!(* * * ? * ?); // Field 'day_of_month' and 'day_of_week' cannot both be unspecified
}
