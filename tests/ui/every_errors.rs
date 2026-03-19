use chronographer::prelude::*;

fn main() {
    every!(1md); // Unexpected suffix "md", did you mean "ms"
    every!(1ms, 3d); // Incorrect time field ordering expected nothing, got "days"
    every!(1s, 3d); // Incorrect time field ordering expected "milliseconds", got "days"
    every!(1m, 3d); // Incorrect time field ordering expected either "milliseconds" or "seconds", got "days"
    every!(3d, 2.5h, 1ms); // Unexpected integer followed after fractional part
    every!(); // Expected time field literals got nothing
    every!(-5d); // Expected a positive integer but got ...
    every!(-2.3h); // Expected a positive float but got ...
    every!(3m,,); // Expected a positive integer or float literal but got something else
    every!(3m 2s, 5ms); // Unexpected a seperator ","
    every!(3m, 2s, 5ms,); // Unexpected a seperator ","
    every!(, 3ms 2d); // Unexpected a seperator ","

    every!(0s); // Exceeded expected range of 0..60 for "seconds" time field, got "0"
    every!(0.0ms); // Exceeded expected range of 0..1000 for "milliseconds" time field, got "0"
    every!(1e2s); // Exceeded expected range of 0..60 for "seconds" time field, got "100"
    every!(2 + 2h); // Expected a positive integer or float literal but got something else
    every!(concat!(1, "s")); // Expected a positive integer or float literal but got something else
    every!(,); // Expected a positive integer or float literal but got something else
    every!(1ŝ); // Unexpected suffix "ŝ"

    every!(1sec); // Unexpected suffix "sec", did you mean "s"
    every!(1min); // Unexpected suffix "min", did you mean "m"
    every!(1hour); // Unexpected suffix "hour", did you mean "h"
    every!(1day); // Unexpected suffix "day", did you mean "d"

    every!(1000ms); // Exceeded expected range of 0..1000 for "milliseconds" time field, got "1000"
    every!(60s); // Exceeded expected range of 0..60 for "seconds" time field, got "60"
    every!(60m); // Exceeded expected range of 0..60 for "minutes" time field, got "60"
    every!(60h); // Exceeded expected range of 0..60 for "hours" time field, got "60"
    every!(32d); // Exceeded expected range of 0..=31 for "days" time field, got "32"

    every!(1m 1m); // Duplicate time field, expected either "milliseconds" or "seconds", got "minutes"

    every!(1s 1m); // Incorrect time field ordering expected "milliseconds", got "minutes"

    every!(1.5m 1s); // Fractional parts are allowed only at the lowest time field

    every!(1m 1s,); // Expected a seperator (,) but got "1m 1s"
    every!(, 1m 1s); // Unexpected a seperator ","

    // Uppercase suffixes (strictly lowercase in implementation)
    every!(1S); // Unexpected suffix "S", did you mean "s"
    every!(1MS); // Unexpected suffix "MS", did you mean "ms"

    // Hex/Binary/Octal literals (not supported by base10_parse)
    every!(0x10s); // Expected a positive integer but got "0x10s"
    every!(0b1010ms); // Expected a positive integer but got "0b1010ms"

    // Mixed separator usage
    every!(1d, 1h 1m); // Expected a seperator (,) but got "1m"

    // Literals with underscores/dots (Forbidden)
    every!(1_000ms); // Expected a positive integer but got "1_000ms"
    every!(.5s); // Expected a positive float literal but got something else
    every!(1.s); // Expected a positive float literal but got something else

    every!(1e1s); // Scientific notation is prohibited in use
    every!(1.2e1s); // Scientific notation is prohibited in use
    every!(1e-1s); // Scientific notation is prohibited in use
}
