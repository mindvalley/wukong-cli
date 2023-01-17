pub mod error;
pub mod table;
pub mod tokenizer;

macro_rules! colored_print {
    () => {
        print!();
    };
    ($($arg:tt)+) => {
        let output = format!("{}", std::format_args!($($arg)+));
        let tokens = $crate::output::tokenizer::OutputTokenizer::tokenize(output);
        for token in tokens {
            print!("{}", token);
        }
    };
}

macro_rules! colored_println {
    () => {
        println!();
    };
    ($($arg:tt)+) => {
        colored_print!($($arg)+);
        println!();
    };
}

// use as export macro
pub(crate) use colored_print;
pub(crate) use colored_println;
