// use scrypto::prelude::*;

// const EXP_LIMIT_192: I192 = I192::from_digits([1000000, 0, 0]);
// const LOG_LIMIT_192: I192 = I192::from_digits([10000, 0, 0]);
// const EXP_X_UPPER_LIMIT: I192 = I192::from_digits([11106511852580896768, 2, 0]);

// /// Calculates the natural logarithm of a given decimal number.
// ///
// /// # Arguments
// ///
// /// * `a` - A decimal number whose natural logarithm is to be calculated.
// ///
// /// # Returns
// ///
// /// * `Decimal` - Natural logarithm of the input number.
// ///
// pub fn log(a: Decimal) -> Decimal {
//     // Convert the decimal input to I192 format
//     let mut a_192: I192 = a.0;
//     // Define a constant for the number 1 in I192 format
//     const ONE_192: I192 = dec!(1).0;
//     const ONE_SIX_192: I192 = dec!(1.6).0;// I192::from_digits([1600000000000000000, 0, 0]);

//     // If the input is less than 1, return the negative log of its inverse
//     if a_192 < ONE_192 {
//         return -log(dec!(1) / a);
//     }

//     let mut result = dec!(0);

//     // Reduce to less than 1.6 by dividing by 2 until the input is less than 1.6
//     while a_192 > ONE_SIX_192 {
//         a_192 /= I192::from(2);
//         result += 1;
//     }

//     // If the input is 1, return 0 as log(1) = 0
//     if a_192 == ONE_192 {
//         return result;
//     }

//     // // Assert that the input is less than or equal to 1.6, as the function does not support larger values
//     // assert!(
//     //     a_192 <= I192::from(1600000000000000000u64),
//     //     "log() only supports values between 0.625 and 1.6."
//     // );

//     // Initialize variables for the calculation
//     let b_192: I192 = a_192 - ONE_192;
//     let mut term_192: I192 = b_192;
//     let mut out_192: I192 = b_192;
//     let mut n: I192 = ONE_192;
//     let mut counter: u8 = 1;

//     // Perform the calculation in a loop
//     for _ in 0..50 {
//         n += ONE_192;
//         counter += 1;

//         term_192 *= b_192;
//         let bit = if counter % 2 == 0 {
//             -term_192 / n
//         } else {
//             term_192 / n
//         };
//         out_192 += bit;

//         term_192 /= ONE_192;

//         // Break the loop if the term is less than the log limit
//         if term_192 < LOG_LIMIT_192 {
//             break;
//         }
//     }

//     // Return the result as a decimal
//     result + Decimal(out_192)
// }

// /// Calculates the exponential of a given decimal number.
// ///
// /// # Arguments
// ///
// /// * `x` - A decimal number whose exponential is to be calculated.
// ///
// /// # Returns
// ///
// /// * `Decimal` - Exponential of the input number.
// ///
// /// # Panics
// ///
// /// * If `x` is less than -48 or greater than 48.
// ///
// pub fn exp(x: Decimal) -> Decimal {
//     // Define a constant for the number 1 in I192 format
//     const ONE_192: I192 = I192::from_digits([1000000000000000000, 0, 0]);
//     // Convert the decimal input to I192 format
//     let x_192: I192 = x.0;

//     // If the input is zero, return 1 as exp(0) = 1
//     if x_192 == I192::ZERO {
//         return Decimal::ONE;
//     }

//     // If the input is negative, invert it and return the inverse of the result at the end
//     let invert = x_192 < I192::ZERO;
//     let x_192 = if invert { -x_192 } else { x_192 };

//     // Assert that the input is less than 48, as the function does not support larger values
//     assert!(
//         x_192 <= EXP_X_UPPER_LIMIT,
//         "exp() only supports values between -48 and 48."
//     );

//     // Initialize variables for the calculation
//     let mut out_192: I192 = ONE_192;
//     let mut term_192: I192 = ONE_192;
//     let mut n: I192 = ONE_192;

//     // Perform the calculation in a loop
//     loop {
//         term_192 = term_192 * x_192 / n;
//         out_192 += term_192;

//         // Break the loop if the term is less than the exp limit
//         if term_192 < EXP_LIMIT_192 {
//             break;
//         }

//         n += ONE_192;
//     }

//     // If the input was negative, return the inverse of the result as a decimal
//     if invert {
//         Decimal::ONE / Decimal(out_192)
//     } else {
//         // Otherwise, return the result as a decimal
//         Decimal(out_192)
//     }
// }

// /// Calculates the power of a given base to a given exponent.
// ///
// /// # Arguments
// ///
// /// * `x` - The base as a decimal number.
// /// * `a` - The exponent as a decimal number.
// ///
// /// # Returns
// ///
// /// * A decimal number representing the base raised to the power of the exponent.
// ///
// /// # Panics
// ///
// /// * If `a` is less than 0.625 or greater than 1.6.
// /// * If `log(a) * x` is less than -48 or greater than 48.
// ///
// pub fn pow(a: Decimal, x: Decimal) -> Decimal {
//     // If the exponent is zero, return 1 as any number raised to the power of 0 is 1
//     if x == Decimal::ZERO {
//         Decimal::ONE
//     } 
//     // If the exponent is one, return the base as any number raised to the power of 1 is the number itself
//     else if x == Decimal::ONE {
//         a
//     } 
//     // For all other cases, calculate the power using the formula a^x = e^(x * ln(a))
//     else {
//         exp(Decimal(x.0 * log(a).0 / Decimal::ONE.0))
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use rand::Rng;

//     #[test]
//     pub fn test() {
//         println!("{}", log(dec!("1.1")));
//         let x = dec!(1) / dec!("1.6");
//         println!("{}", x);//-0.470003629245770777 0.470003629245770777

//         let y = dec!(48) / log(dec!("1.6"));
//         println!("{}", y);
//     }

//     fn assert_almost_equal(a: Decimal, b: Decimal, d: Decimal) {
//         if a.is_zero() {
//             assert!(
//                 b.is_zero(),
//                 "a is zero, but b is not zero. a: {:?}, b: {:?}",
//                 a,
//                 b
//             );
//         } else {
//             assert!(
//                 (a - b).checked_abs().unwrap() / a < d,
//                 "The difference is not less than d. a: {:?}, b: {:?}",
//                 a,
//                 b
//             );
//         }
//     }

//     #[test]
//     fn test_log_one() {
//         // ARRANGE
//         let x = Decimal::ONE;

//         // ACT
//         let y = log(x);

//         // ASSERT
//         assert_eq!(y, Decimal::ZERO);
//     }

//     #[test]
//     fn test_ln_01() {
//         // ARRANGE
//         let x_f64: f64 = 1.6;
//         let x = Decimal::try_from(x_f64.to_string()).unwrap();

//         // ACT
//         let y = log(x);
//         let y_true = Decimal::try_from(x_f64.ln().to_string()).unwrap();

//         // println!(
//         //     "x: {} y: {} y_true {}, error: {}",
//         //     x,
//         //     y,
//         //     y_true,
//         //     (y - y_true).abs() / y_true
//         // );

//         // ASSERT
//         assert_almost_equal(y, y_true, dec!("0.00000001"));
//     }

//     #[test]
//     fn test_ln_around_one() {
//         let mut rng = rand::thread_rng();
//         for _ in 0..50000 {
//             // ARRANGE
//             let x_f64: f64 = rng.gen_range(0.62500..1.60000);
//             match Decimal::try_from(x_f64.to_string()) {
//                 Ok(x) => {
//                     // ACT
//                     let y = log(x);
//                     match Decimal::try_from(x_f64.ln().to_string()) {
//                         Ok(y_true) => {
//                             // ASSERT
//                             assert_almost_equal(y, y_true, dec!("0.00000001"));
//                         }
//                         Err(_) => {}
//                     }
//                 }
//                 Err(_) => {}
//             }
//         }
//     }

//     #[test]
//     fn test_exp_zero() {
//         // ARRANGE
//         let x = Decimal::ZERO;

//         // ACT
//         let y = exp(x);

//         // ASSERT
//         assert_eq!(y, Decimal::ONE);
//     }

//     #[test]
//     #[should_panic]
//     fn test_exp_large_number() {
//         // ARRANGE
//         let x = dec!("48.000000000000000001");

//         // ASSERT - panic
//         let _y = exp(x);
//     }

//     #[test]
//     #[should_panic]
//     fn test_exp_large_negative_number() {
//         // ARRANGE
//         let x = dec!("-48.000000000000000001");

//         // ASSERT - panic
//         let _y = exp(x);
//     }

//     #[test]
//     fn test_exp_large_number_below_limit_succeeds() {
//         // ARRANGE
//         let x = dec!("48");

//         // ACT - ASSERT
//         let _y = exp(x);
//     }

//     #[test]
//     fn test_exp_large_negative_number_below_limit_succeeds() {
//         // ARRANGE
//         let x = dec!("-48");

//         // ACT - ASSERT
//         let _y = exp(x);
//     }

//     #[test]
//     fn test_exp_around_zero() {
//         let mut rng = rand::thread_rng();
//         for _ in 0..1000 {
//             // ARRANGE
//             let x_f64: f64 = rng.gen_range(-0.10000..0.10000);
//             match Decimal::try_from(x_f64.to_string()) {
//                 Ok(x) => {
//                     // ACT
//                     let y = exp(x);
//                     let y_true = Decimal::try_from(x_f64.exp().to_string()).unwrap();

//                     // println!("x: {} y: {} y_true {}", x, y, y_true);

//                     // ASSERT
//                     assert_almost_equal(y, y_true, dec!("0.0000000001"));
//                 }
//                 Err(_) => {}
//             }
//         }
//     }

//     #[test]
//     fn test_exp_around_one() {
//         let mut rng = rand::thread_rng();
//         for _ in 0..1000 {
//             // ARRANGE
//             let x_f64: f64 = rng.gen_range(0.90000..1.10000);
//             match Decimal::try_from(x_f64.to_string()) {
//                 Ok(x) => {
//                     // ACT
//                     let y = exp(x);
//                     let y_true = Decimal::try_from(x_f64.exp().to_string()).unwrap();

//                     // println!("x: {} y: {} y_true {}", x, y, y_true);

//                     // ASSERT
//                     assert_almost_equal(y, y_true, dec!("0.0000000001"));
//                 }
//                 Err(_) => {}
//             }
//         }
//     }

//     #[test]
//     fn test_exp_invert() {
//         let mut rng = rand::thread_rng();
//         for _ in 0..1000 {
//             // ARRANGE
//             let x_f64: f64 = rng.gen_range(0.0..48.0);
//             match Decimal::try_from(x_f64.to_string()) {
//                 Ok(x) => {
//                     println!("x: {}", x);
//                     // ACT
//                     let y = exp(x);
//                     let y_invert = exp(-x);
//                     let y_true = Decimal::try_from(x_f64.exp().to_string()).unwrap();

//                     // println!(
//                     //     "x: {} y: {} y_invert: {}, y_true {}",
//                     //     x, y, y_invert, y_true
//                     // );

//                     // ASSERT
//                     assert_almost_equal(y, y_true, dec!("0.0000000001"));
//                     assert_almost_equal(y_invert, Decimal::ONE / y, dec!("0.0000000001"));
//                 }
//                 Err(_) => {}
//             }
//         }
//     }
// }