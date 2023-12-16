#[macro_export]
macro_rules! require_instruction_eq {
    ($value1: expr, $value2: expr, $value3: expr, $error_code:expr $(,)?) => {
        use crate::require_discriminator_eq;
        require_keys_eq!($value1.program_id, $value2, $error_code);
        require_discriminator_eq!($value1, $value3, $error_code);
    };
    ($value1: expr, $value2: expr, $value3: expr $(,)?) => {
        require_keys_eq!($value1.program_id, $value2);
        require_discriminator_eq!($value1, $value3);
    };
}