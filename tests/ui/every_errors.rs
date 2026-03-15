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
}
