#[macro_export]
macro_rules! require_discriminator_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        if !$value1.data[0..8].eq($value2.as_slice()) {
            return Err(error!($error_code).with_values((
                format_args!(
                    "{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}", 
                    $value1.data[0],
                    $value1.data[1],
                    $value1.data[2],
                    $value1.data[3],
                    $value1.data[4],
                    $value1.data[5],
                    $value1.data[6],
                    $value1.data[7]
                ),
                format_args!(
                    "{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}", 
                    $value2[0],
                    $value2[1],
                    $value2[2],
                    $value2[3],
                    $value2[4],
                    $value2[5],
                    $value2[6],
                    $value2[7]
                )) 
            ));
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if !$value1.data[0..8].eq($value2.as_slice()) {
            return Err(error!(anchor_lang::error::ErrorCode::InstructionDidNotDeserialize));
        }
    };
}