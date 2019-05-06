// Copyright 2014-2016 bluss and ndarray developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use super::{ArrayBase, Axis, Data, Dimension, Ix, NdProducer};
use crate::aliases::Ix1;
use std::fmt;

const PRINT_ELEMENTS_LIMIT: Ix = 3;

fn format_1d_array<A, S, F>(
    view: &ArrayBase<S, Ix1>,
    f: &mut fmt::Formatter<'_>,
    mut format: F,
    limit: Ix,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result,
    S: Data<Elem = A>,
{
    let to_be_printed = to_be_printed(view.len(), limit);

    let n_to_be_printed = to_be_printed.len();

    write!(f, "[")?;
    for (j, index) in to_be_printed.into_iter().enumerate() {
        match index {
            PrintableCell::ElementIndex(i) => {
                format(&view[i], f)?;
                if j != n_to_be_printed - 1 {
                    write!(f, ", ")?;
                }
            }
            PrintableCell::Ellipses => write!(f, "..., ")?,
        }
    }
    write!(f, "]")?;
    Ok(())
}

enum PrintableCell {
    ElementIndex(usize),
    Ellipses,
}

// Returns what indexes should be printed for a certain axis.
// If the axis is longer than 2 * limit, a `Ellipses` is inserted
// where indexes are being omitted.
fn to_be_printed(length: usize, limit: usize) -> Vec<PrintableCell> {
    if length <= 2 * limit {
        (0..length).map(PrintableCell::ElementIndex).collect()
    } else {
        let mut v: Vec<PrintableCell> = (0..limit).map(PrintableCell::ElementIndex).collect();
        v.push(PrintableCell::Ellipses);
        v.extend((length - limit..length).map(PrintableCell::ElementIndex));
        v
    }
}

fn format_array<A, S, D, F>(
    view: &ArrayBase<S, D>,
    f: &mut fmt::Formatter<'_>,
    mut format: F,
    limit: Ix,
    depth: usize,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result + Clone,
    D: Dimension,
    S: Data<Elem = A>,
{
    // If any of the axes has 0 length, we return the same empty array representation
    // e.g. [[]] for 2-d arrays
    if view.shape().iter().any(|&x| x == 0) {
        write!(f, "{}{}", "[".repeat(view.ndim()), "]".repeat(view.ndim()))?;
        return Ok(());
    }
    match view.shape() {
        // If it's 0 dimensional, we just print out the scalar
        [] => format(view.iter().next().unwrap(), f)?,
        // We delegate 1-dimensional arrays to a specialized function
        [_] => format_1d_array(
            &view.view().into_dimensionality::<Ix1>().unwrap(),
            f,
            format,
            limit,
        )?,
        // For n-dimensional arrays, we proceed recursively
        shape => {
            // Cast into a dynamically dimensioned view
            // This is required to be able to use `index_axis`
            let view = view.view().into_dyn();
            // We start by checking what indexes from the first axis should be printed
            // We put a `None` in the middle if we are omitting elements
            let to_be_printed = to_be_printed(shape[0], limit);

            let n_to_be_printed = to_be_printed.len();

            let blank_lines = "\n".repeat(shape.len() - 2);
            let indent = " ".repeat(depth + 1);

            write!(f, "[")?;
            for (j, index) in to_be_printed.into_iter().enumerate() {
                match index {
                    PrintableCell::ElementIndex(i) => {
                        // Indent all but the first line.
                        if j != 0 {
                            write!(f, "{}", indent)?;
                        }
                        // Proceed recursively with the (n-1)-dimensional slice
                        format_array(
                            &view.index_axis(Axis(0), i),
                            f,
                            format.clone(),
                            limit,
                            depth + 1,
                        )?;
                        // We need to add a separator after each slice,
                        // apart from the last one
                        if j != n_to_be_printed - 1 {
                            write!(f, ",\n{}", blank_lines)?
                        }
                    }
                    PrintableCell::Ellipses => write!(f, "{}...,\n{}", indent, blank_lines)?,
                }
            }
            write!(f, "]")?;
        }
    }
    Ok(())
}

// NOTE: We can impl other fmt traits here
/// Format the array using `Display` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Display, S, D: Dimension> fmt::Display for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)
    }
}

/// Format the array using `Debug` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Debug, S, D: Dimension> fmt::Debug for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Add extra information for Debug
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)?;
        write!(
            f,
            " shape={:?}, strides={:?}, layout={:?}",
            self.shape(),
            self.strides(),
            layout = self.view().layout()
        )?;
        match D::NDIM {
            Some(ndim) => write!(f, ", const ndim={}", ndim)?,
            None => write!(f, ", dynamic ndim={}", self.ndim())?,
        }
        Ok(())
    }
}

/// Format the array using `LowerExp` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::LowerExp, S, D: Dimension> fmt::LowerExp for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)
    }
}

/// Format the array using `UpperExp` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::UpperExp, S, D: Dimension> fmt::UpperExp for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)
    }
}
/// Format the array using `LowerHex` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::LowerHex, S, D: Dimension> fmt::LowerHex for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)
    }
}

/// Format the array using `Binary` and apply the formatting parameters used
/// to each element.
///
/// The array is shown in multiline style.
impl<'a, A: fmt::Binary, S, D: Dimension> fmt::Binary for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_array(self, f, <_>::fmt, PRINT_ELEMENTS_LIMIT, 0)
    }
}

#[cfg(test)]
mod formatting_with_omit {
    use super::*;
    use crate::prelude::*;

    fn print_output_diff(expected: &str, actual: &str) {
        println!("Expected output:\n{}\nActual output:\n{}", expected, actual);
    }

    #[test]
    fn empty_arrays() {
        let a: Array2<u32> = arr2(&[[], []]);
        let actual_output = format!("{}", a);
        let expected_output = String::from("[[]]");
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn zero_length_axes() {
        let a = Array3::<f32>::zeros((3, 0, 4));
        let actual_output = format!("{}", a);
        let expected_output = String::from("[[[]]]");
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn dim_0() {
        let element = 12;
        let a = arr0(element);
        let actual_output = format!("{}", a);
        let expected_output = format!("{}", element);
        print_output_diff(&expected_output, &actual_output);
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn dim_1() {
        let overflow: usize = 5;
        let a = Array1::from_elem((PRINT_ELEMENTS_LIMIT * 2 + overflow,), 1);
        let mut expected_output = String::from("[");
        a.iter()
            .take(PRINT_ELEMENTS_LIMIT)
            .for_each(|elem| expected_output.push_str(format!("{}, ", elem).as_str()));
        expected_output.push_str("...");
        a.iter()
            .skip(PRINT_ELEMENTS_LIMIT + overflow)
            .for_each(|elem| expected_output.push_str(format!(", {}", elem).as_str()));
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_last_axis_overflow() {
        let overflow: usize = 3;
        let a = Array2::from_elem(
            (PRINT_ELEMENTS_LIMIT, PRINT_ELEMENTS_LIMIT * 2 + overflow),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i < PRINT_ELEMENTS_LIMIT - 1 {
                "],\n "
            } else {
                "]"
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_non_last_axis_overflow() {
        let overflow: usize = 5;
        let a = Array2::from_elem(
            (PRINT_ELEMENTS_LIMIT * 2 + overflow, PRINT_ELEMENTS_LIMIT),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str("],\n ");
        }
        expected_output.push_str("...,\n ");
        for i in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i == PRINT_ELEMENTS_LIMIT * 2 + overflow - 1 {
                "]"
            } else {
                "],\n "
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_2_multi_directional_overflow() {
        let overflow: usize = 5;
        let a = Array2::from_elem(
            (
                PRINT_ELEMENTS_LIMIT * 2 + overflow,
                PRINT_ELEMENTS_LIMIT * 2 + overflow,
            ),
            1,
        );
        let mut expected_output = String::from("[");

        for i in 0..PRINT_ELEMENTS_LIMIT {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str("],\n ");
        }
        expected_output.push_str("...,\n ");
        for i in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
            expected_output.push_str(format!("[{}", a[(i, 0)]).as_str());
            for j in 1..PRINT_ELEMENTS_LIMIT {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(", ...");
            for j in PRINT_ELEMENTS_LIMIT + overflow..PRINT_ELEMENTS_LIMIT * 2 + overflow {
                expected_output.push_str(format!(", {}", a[(i, j)]).as_str());
            }
            expected_output.push_str(if i == PRINT_ELEMENTS_LIMIT * 2 + overflow - 1 {
                "]"
            } else {
                "],\n "
            });
        }
        expected_output.push(']');
        let actual_output = format!("{}", a);

        print_output_diff(&expected_output, &actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn dim_3_overflow_all() {
        let a = Array3::from_shape_fn((20, 10, 7), |(i, j, k)| {
            1000. + (100. * ((i as f64).sqrt() + (j as f64).sin() + k as f64)).round() / 100.
        });
        // Generated using NumPy with `np.set_printoptions(suppress=True, floatmode='maxprec_equal')`.
        let correct = "\
[[[1000.00, 1001.00, 1002.00, ..., 1004.00, 1005.00, 1006.00],
  [1000.84, 1001.84, 1002.84, ..., 1004.84, 1005.84, 1006.84],
  [1000.91, 1001.91, 1002.91, ..., 1004.91, 1005.91, 1006.91],
  ...,
  [1000.66, 1001.66, 1002.66, ..., 1004.66, 1005.66, 1006.66],
  [1000.99, 1001.99, 1002.99, ..., 1004.99, 1005.99, 1006.99],
  [1000.41, 1001.41, 1002.41, ..., 1004.41, 1005.41, 1006.41]],

 [[1001.00, 1002.00, 1003.00, ..., 1005.00, 1006.00, 1007.00],
  [1001.84, 1002.84, 1003.84, ..., 1005.84, 1006.84, 1007.84],
  [1001.91, 1002.91, 1003.91, ..., 1005.91, 1006.91, 1007.91],
  ...,
  [1001.66, 1002.66, 1003.66, ..., 1005.66, 1006.66, 1007.66],
  [1001.99, 1002.99, 1003.99, ..., 1005.99, 1006.99, 1007.99],
  [1001.41, 1002.41, 1003.41, ..., 1005.41, 1006.41, 1007.41]],

 [[1001.41, 1002.41, 1003.41, ..., 1005.41, 1006.41, 1007.41],
  [1002.26, 1003.26, 1004.26, ..., 1006.26, 1007.26, 1008.26],
  [1002.32, 1003.32, 1004.32, ..., 1006.32, 1007.32, 1008.32],
  ...,
  [1002.07, 1003.07, 1004.07, ..., 1006.07, 1007.07, 1008.07],
  [1002.40, 1003.40, 1004.40, ..., 1006.40, 1007.40, 1008.40],
  [1001.83, 1002.83, 1003.83, ..., 1005.83, 1006.83, 1007.83]],

 ...,

 [[1004.12, 1005.12, 1006.12, ..., 1008.12, 1009.12, 1010.12],
  [1004.96, 1005.96, 1006.96, ..., 1008.96, 1009.96, 1010.96],
  [1005.03, 1006.03, 1007.03, ..., 1009.03, 1010.03, 1011.03],
  ...,
  [1004.78, 1005.78, 1006.78, ..., 1008.78, 1009.78, 1010.78],
  [1005.11, 1006.11, 1007.11, ..., 1009.11, 1010.11, 1011.11],
  [1004.54, 1005.54, 1006.54, ..., 1008.54, 1009.54, 1010.54]],

 [[1004.24, 1005.24, 1006.24, ..., 1008.24, 1009.24, 1010.24],
  [1005.08, 1006.08, 1007.08, ..., 1009.08, 1010.08, 1011.08],
  [1005.15, 1006.15, 1007.15, ..., 1009.15, 1010.15, 1011.15],
  ...,
  [1004.90, 1005.90, 1006.90, ..., 1008.90, 1009.90, 1010.90],
  [1005.23, 1006.23, 1007.23, ..., 1009.23, 1010.23, 1011.23],
  [1004.65, 1005.65, 1006.65, ..., 1008.65, 1009.65, 1010.65]],

 [[1004.36, 1005.36, 1006.36, ..., 1008.36, 1009.36, 1010.36],
  [1005.20, 1006.20, 1007.20, ..., 1009.20, 1010.20, 1011.20],
  [1005.27, 1006.27, 1007.27, ..., 1009.27, 1010.27, 1011.27],
  ...,
  [1005.02, 1006.02, 1007.02, ..., 1009.02, 1010.02, 1011.02],
  [1005.35, 1006.35, 1007.35, ..., 1009.35, 1010.35, 1011.35],
  [1004.77, 1005.77, 1006.77, ..., 1008.77, 1009.77, 1010.77]]]";
        assert_eq!(format!("{:.2}", a), correct);
    }

    #[test]
    fn dim_4_overflow_all() {
        let a = Array4::from_shape_fn((20, 10, 7, 8), |(i, j, k, l)| {
            (100. * ((i as f64).sqrt() + (j as f64).exp() + (k as f64).sin() + l as f64)).round()
                / 100.
                + 1000.
        });
        // Generated using NumPy with `np.set_printoptions(suppress=True, floatmode='maxprec_equal')`.
        let correct = "\
[[[[1001.00, 1002.00, 1003.00, ..., 1006.00, 1007.00, 1008.00],
   [1001.84, 1002.84, 1003.84, ..., 1006.84, 1007.84, 1008.84],
   [1001.91, 1002.91, 1003.91, ..., 1006.91, 1007.91, 1008.91],
   ...,
   [1000.24, 1001.24, 1002.24, ..., 1005.24, 1006.24, 1007.24],
   [1000.04, 1001.04, 1002.04, ..., 1005.04, 1006.04, 1007.04],
   [1000.72, 1001.72, 1002.72, ..., 1005.72, 1006.72, 1007.72]],

  [[1002.72, 1003.72, 1004.72, ..., 1007.72, 1008.72, 1009.72],
   [1003.56, 1004.56, 1005.56, ..., 1008.56, 1009.56, 1010.56],
   [1003.63, 1004.63, 1005.63, ..., 1008.63, 1009.63, 1010.63],
   ...,
   [1001.96, 1002.96, 1003.96, ..., 1006.96, 1007.96, 1008.96],
   [1001.76, 1002.76, 1003.76, ..., 1006.76, 1007.76, 1008.76],
   [1002.44, 1003.44, 1004.44, ..., 1007.44, 1008.44, 1009.44]],

  [[1007.39, 1008.39, 1009.39, ..., 1012.39, 1013.39, 1014.39],
   [1008.23, 1009.23, 1010.23, ..., 1013.23, 1014.23, 1015.23],
   [1008.30, 1009.30, 1010.30, ..., 1013.30, 1014.30, 1015.30],
   ...,
   [1006.63, 1007.63, 1008.63, ..., 1011.63, 1012.63, 1013.63],
   [1006.43, 1007.43, 1008.43, ..., 1011.43, 1012.43, 1013.43],
   [1007.11, 1008.11, 1009.11, ..., 1012.11, 1013.11, 1014.11]],

  ...,

  [[2096.63, 2097.63, 2098.63, ..., 2101.63, 2102.63, 2103.63],
   [2097.47, 2098.47, 2099.47, ..., 2102.47, 2103.47, 2104.47],
   [2097.54, 2098.54, 2099.54, ..., 2102.54, 2103.54, 2104.54],
   ...,
   [2095.88, 2096.88, 2097.88, ..., 2100.88, 2101.88, 2102.88],
   [2095.67, 2096.67, 2097.67, ..., 2100.67, 2101.67, 2102.67],
   [2096.35, 2097.35, 2098.35, ..., 2101.35, 2102.35, 2103.35]],

  [[3980.96, 3981.96, 3982.96, ..., 3985.96, 3986.96, 3987.96],
   [3981.80, 3982.80, 3983.80, ..., 3986.80, 3987.80, 3988.80],
   [3981.87, 3982.87, 3983.87, ..., 3986.87, 3987.87, 3988.87],
   ...,
   [3980.20, 3981.20, 3982.20, ..., 3985.20, 3986.20, 3987.20],
   [3980.00, 3981.00, 3982.00, ..., 3985.00, 3986.00, 3987.00],
   [3980.68, 3981.68, 3982.68, ..., 3985.68, 3986.68, 3987.68]],

  [[9103.08, 9104.08, 9105.08, ..., 9108.08, 9109.08, 9110.08],
   [9103.93, 9104.93, 9105.93, ..., 9108.93, 9109.93, 9110.93],
   [9103.99, 9104.99, 9105.99, ..., 9108.99, 9109.99, 9110.99],
   ...,
   [9102.33, 9103.33, 9104.33, ..., 9107.33, 9108.33, 9109.33],
   [9102.13, 9103.13, 9104.13, ..., 9107.13, 9108.13, 9109.13],
   [9102.80, 9103.80, 9104.80, ..., 9107.80, 9108.80, 9109.80]]],


 [[[1002.00, 1003.00, 1004.00, ..., 1007.00, 1008.00, 1009.00],
   [1002.84, 1003.84, 1004.84, ..., 1007.84, 1008.84, 1009.84],
   [1002.91, 1003.91, 1004.91, ..., 1007.91, 1008.91, 1009.91],
   ...,
   [1001.24, 1002.24, 1003.24, ..., 1006.24, 1007.24, 1008.24],
   [1001.04, 1002.04, 1003.04, ..., 1006.04, 1007.04, 1008.04],
   [1001.72, 1002.72, 1003.72, ..., 1006.72, 1007.72, 1008.72]],

  [[1003.72, 1004.72, 1005.72, ..., 1008.72, 1009.72, 1010.72],
   [1004.56, 1005.56, 1006.56, ..., 1009.56, 1010.56, 1011.56],
   [1004.63, 1005.63, 1006.63, ..., 1009.63, 1010.63, 1011.63],
   ...,
   [1002.96, 1003.96, 1004.96, ..., 1007.96, 1008.96, 1009.96],
   [1002.76, 1003.76, 1004.76, ..., 1007.76, 1008.76, 1009.76],
   [1003.44, 1004.44, 1005.44, ..., 1008.44, 1009.44, 1010.44]],

  [[1008.39, 1009.39, 1010.39, ..., 1013.39, 1014.39, 1015.39],
   [1009.23, 1010.23, 1011.23, ..., 1014.23, 1015.23, 1016.23],
   [1009.30, 1010.30, 1011.30, ..., 1014.30, 1015.30, 1016.30],
   ...,
   [1007.63, 1008.63, 1009.63, ..., 1012.63, 1013.63, 1014.63],
   [1007.43, 1008.43, 1009.43, ..., 1012.43, 1013.43, 1014.43],
   [1008.11, 1009.11, 1010.11, ..., 1013.11, 1014.11, 1015.11]],

  ...,

  [[2097.63, 2098.63, 2099.63, ..., 2102.63, 2103.63, 2104.63],
   [2098.47, 2099.47, 2100.47, ..., 2103.47, 2104.47, 2105.47],
   [2098.54, 2099.54, 2100.54, ..., 2103.54, 2104.54, 2105.54],
   ...,
   [2096.88, 2097.88, 2098.88, ..., 2101.88, 2102.88, 2103.88],
   [2096.67, 2097.67, 2098.67, ..., 2101.67, 2102.67, 2103.67],
   [2097.35, 2098.35, 2099.35, ..., 2102.35, 2103.35, 2104.35]],

  [[3981.96, 3982.96, 3983.96, ..., 3986.96, 3987.96, 3988.96],
   [3982.80, 3983.80, 3984.80, ..., 3987.80, 3988.80, 3989.80],
   [3982.87, 3983.87, 3984.87, ..., 3987.87, 3988.87, 3989.87],
   ...,
   [3981.20, 3982.20, 3983.20, ..., 3986.20, 3987.20, 3988.20],
   [3981.00, 3982.00, 3983.00, ..., 3986.00, 3987.00, 3988.00],
   [3981.68, 3982.68, 3983.68, ..., 3986.68, 3987.68, 3988.68]],

  [[9104.08, 9105.08, 9106.08, ..., 9109.08, 9110.08, 9111.08],
   [9104.93, 9105.93, 9106.93, ..., 9109.93, 9110.93, 9111.93],
   [9104.99, 9105.99, 9106.99, ..., 9109.99, 9110.99, 9111.99],
   ...,
   [9103.33, 9104.33, 9105.33, ..., 9108.33, 9109.33, 9110.33],
   [9103.13, 9104.13, 9105.13, ..., 9108.13, 9109.13, 9110.13],
   [9103.80, 9104.80, 9105.80, ..., 9108.80, 9109.80, 9110.80]]],


 [[[1002.41, 1003.41, 1004.41, ..., 1007.41, 1008.41, 1009.41],
   [1003.26, 1004.26, 1005.26, ..., 1008.26, 1009.26, 1010.26],
   [1003.32, 1004.32, 1005.32, ..., 1008.32, 1009.32, 1010.32],
   ...,
   [1001.66, 1002.66, 1003.66, ..., 1006.66, 1007.66, 1008.66],
   [1001.46, 1002.46, 1003.46, ..., 1006.46, 1007.46, 1008.46],
   [1002.13, 1003.13, 1004.13, ..., 1007.13, 1008.13, 1009.13]],

  [[1004.13, 1005.13, 1006.13, ..., 1009.13, 1010.13, 1011.13],
   [1004.97, 1005.97, 1006.97, ..., 1009.97, 1010.97, 1011.97],
   [1005.04, 1006.04, 1007.04, ..., 1010.04, 1011.04, 1012.04],
   ...,
   [1003.38, 1004.38, 1005.38, ..., 1008.38, 1009.38, 1010.38],
   [1003.17, 1004.17, 1005.17, ..., 1008.17, 1009.17, 1010.17],
   [1003.85, 1004.85, 1005.85, ..., 1008.85, 1009.85, 1010.85]],

  [[1008.80, 1009.80, 1010.80, ..., 1013.80, 1014.80, 1015.80],
   [1009.64, 1010.64, 1011.64, ..., 1014.64, 1015.64, 1016.64],
   [1009.71, 1010.71, 1011.71, ..., 1014.71, 1015.71, 1016.71],
   ...,
   [1008.05, 1009.05, 1010.05, ..., 1013.05, 1014.05, 1015.05],
   [1007.84, 1008.84, 1009.84, ..., 1012.84, 1013.84, 1014.84],
   [1008.52, 1009.52, 1010.52, ..., 1013.52, 1014.52, 1015.52]],

  ...,

  [[2098.05, 2099.05, 2100.05, ..., 2103.05, 2104.05, 2105.05],
   [2098.89, 2099.89, 2100.89, ..., 2103.89, 2104.89, 2105.89],
   [2098.96, 2099.96, 2100.96, ..., 2103.96, 2104.96, 2105.96],
   ...,
   [2097.29, 2098.29, 2099.29, ..., 2102.29, 2103.29, 2104.29],
   [2097.09, 2098.09, 2099.09, ..., 2102.09, 2103.09, 2104.09],
   [2097.77, 2098.77, 2099.77, ..., 2102.77, 2103.77, 2104.77]],

  [[3982.37, 3983.37, 3984.37, ..., 3987.37, 3988.37, 3989.37],
   [3983.21, 3984.21, 3985.21, ..., 3988.21, 3989.21, 3990.21],
   [3983.28, 3984.28, 3985.28, ..., 3988.28, 3989.28, 3990.28],
   ...,
   [3981.62, 3982.62, 3983.62, ..., 3986.62, 3987.62, 3988.62],
   [3981.41, 3982.41, 3983.41, ..., 3986.41, 3987.41, 3988.41],
   [3982.09, 3983.09, 3984.09, ..., 3987.09, 3988.09, 3989.09]],

  [[9104.50, 9105.50, 9106.50, ..., 9109.50, 9110.50, 9111.50],
   [9105.34, 9106.34, 9107.34, ..., 9110.34, 9111.34, 9112.34],
   [9105.41, 9106.41, 9107.41, ..., 9110.41, 9111.41, 9112.41],
   ...,
   [9103.74, 9104.74, 9105.74, ..., 9108.74, 9109.74, 9110.74],
   [9103.54, 9104.54, 9105.54, ..., 9108.54, 9109.54, 9110.54],
   [9104.22, 9105.22, 9106.22, ..., 9109.22, 9110.22, 9111.22]]],


 ...,


 [[[1005.12, 1006.12, 1007.12, ..., 1010.12, 1011.12, 1012.12],
   [1005.96, 1006.96, 1007.96, ..., 1010.96, 1011.96, 1012.96],
   [1006.03, 1007.03, 1008.03, ..., 1011.03, 1012.03, 1013.03],
   ...,
   [1004.37, 1005.37, 1006.37, ..., 1009.37, 1010.37, 1011.37],
   [1004.16, 1005.16, 1006.16, ..., 1009.16, 1010.16, 1011.16],
   [1004.84, 1005.84, 1006.84, ..., 1009.84, 1010.84, 1011.84]],

  [[1006.84, 1007.84, 1008.84, ..., 1011.84, 1012.84, 1013.84],
   [1007.68, 1008.68, 1009.68, ..., 1012.68, 1013.68, 1014.68],
   [1007.75, 1008.75, 1009.75, ..., 1012.75, 1013.75, 1014.75],
   ...,
   [1006.08, 1007.08, 1008.08, ..., 1011.08, 1012.08, 1013.08],
   [1005.88, 1006.88, 1007.88, ..., 1010.88, 1011.88, 1012.88],
   [1006.56, 1007.56, 1008.56, ..., 1011.56, 1012.56, 1013.56]],

  [[1011.51, 1012.51, 1013.51, ..., 1016.51, 1017.51, 1018.51],
   [1012.35, 1013.35, 1014.35, ..., 1017.35, 1018.35, 1019.35],
   [1012.42, 1013.42, 1014.42, ..., 1017.42, 1018.42, 1019.42],
   ...,
   [1010.76, 1011.76, 1012.76, ..., 1015.76, 1016.76, 1017.76],
   [1010.55, 1011.55, 1012.55, ..., 1015.55, 1016.55, 1017.55],
   [1011.23, 1012.23, 1013.23, ..., 1016.23, 1017.23, 1018.23]],

  ...,

  [[2100.76, 2101.76, 2102.76, ..., 2105.76, 2106.76, 2107.76],
   [2101.60, 2102.60, 2103.60, ..., 2106.60, 2107.60, 2108.60],
   [2101.67, 2102.67, 2103.67, ..., 2106.67, 2107.67, 2108.67],
   ...,
   [2100.00, 2101.00, 2102.00, ..., 2105.00, 2106.00, 2107.00],
   [2099.80, 2100.80, 2101.80, ..., 2104.80, 2105.80, 2106.80],
   [2100.48, 2101.48, 2102.48, ..., 2105.48, 2106.48, 2107.48]],

  [[3985.08, 3986.08, 3987.08, ..., 3990.08, 3991.08, 3992.08],
   [3985.92, 3986.92, 3987.92, ..., 3990.92, 3991.92, 3992.92],
   [3985.99, 3986.99, 3987.99, ..., 3990.99, 3991.99, 3992.99],
   ...,
   [3984.32, 3985.32, 3986.32, ..., 3989.32, 3990.32, 3991.32],
   [3984.12, 3985.12, 3986.12, ..., 3989.12, 3990.12, 3991.12],
   [3984.80, 3985.80, 3986.80, ..., 3989.80, 3990.80, 3991.80]],

  [[9107.21, 9108.21, 9109.21, ..., 9112.21, 9113.21, 9114.21],
   [9108.05, 9109.05, 9110.05, ..., 9113.05, 9114.05, 9115.05],
   [9108.12, 9109.12, 9110.12, ..., 9113.12, 9114.12, 9115.12],
   ...,
   [9106.45, 9107.45, 9108.45, ..., 9111.45, 9112.45, 9113.45],
   [9106.25, 9107.25, 9108.25, ..., 9111.25, 9112.25, 9113.25],
   [9106.93, 9107.93, 9108.93, ..., 9111.93, 9112.93, 9113.93]]],


 [[[1005.24, 1006.24, 1007.24, ..., 1010.24, 1011.24, 1012.24],
   [1006.08, 1007.08, 1008.08, ..., 1011.08, 1012.08, 1013.08],
   [1006.15, 1007.15, 1008.15, ..., 1011.15, 1012.15, 1013.15],
   ...,
   [1004.49, 1005.49, 1006.49, ..., 1009.49, 1010.49, 1011.49],
   [1004.28, 1005.28, 1006.28, ..., 1009.28, 1010.28, 1011.28],
   [1004.96, 1005.96, 1006.96, ..., 1009.96, 1010.96, 1011.96]],

  [[1006.96, 1007.96, 1008.96, ..., 1011.96, 1012.96, 1013.96],
   [1007.80, 1008.80, 1009.80, ..., 1012.80, 1013.80, 1014.80],
   [1007.87, 1008.87, 1009.87, ..., 1012.87, 1013.87, 1014.87],
   ...,
   [1006.20, 1007.20, 1008.20, ..., 1011.20, 1012.20, 1013.20],
   [1006.00, 1007.00, 1008.00, ..., 1011.00, 1012.00, 1013.00],
   [1006.68, 1007.68, 1008.68, ..., 1011.68, 1012.68, 1013.68]],

  [[1011.63, 1012.63, 1013.63, ..., 1016.63, 1017.63, 1018.63],
   [1012.47, 1013.47, 1014.47, ..., 1017.47, 1018.47, 1019.47],
   [1012.54, 1013.54, 1014.54, ..., 1017.54, 1018.54, 1019.54],
   ...,
   [1010.87, 1011.87, 1012.87, ..., 1015.87, 1016.87, 1017.87],
   [1010.67, 1011.67, 1012.67, ..., 1015.67, 1016.67, 1017.67],
   [1011.35, 1012.35, 1013.35, ..., 1016.35, 1017.35, 1018.35]],

  ...,

  [[2100.88, 2101.88, 2102.88, ..., 2105.88, 2106.88, 2107.88],
   [2101.72, 2102.72, 2103.72, ..., 2106.72, 2107.72, 2108.72],
   [2101.79, 2102.79, 2103.79, ..., 2106.79, 2107.79, 2108.79],
   ...,
   [2100.12, 2101.12, 2102.12, ..., 2105.12, 2106.12, 2107.12],
   [2099.92, 2100.92, 2101.92, ..., 2104.92, 2105.92, 2106.92],
   [2100.60, 2101.60, 2102.60, ..., 2105.60, 2106.60, 2107.60]],

  [[3985.20, 3986.20, 3987.20, ..., 3990.20, 3991.20, 3992.20],
   [3986.04, 3987.04, 3988.04, ..., 3991.04, 3992.04, 3993.04],
   [3986.11, 3987.11, 3988.11, ..., 3991.11, 3992.11, 3993.11],
   ...,
   [3984.44, 3985.44, 3986.44, ..., 3989.44, 3990.44, 3991.44],
   [3984.24, 3985.24, 3986.24, ..., 3989.24, 3990.24, 3991.24],
   [3984.92, 3985.92, 3986.92, ..., 3989.92, 3990.92, 3991.92]],

  [[9107.33, 9108.33, 9109.33, ..., 9112.33, 9113.33, 9114.33],
   [9108.17, 9109.17, 9110.17, ..., 9113.17, 9114.17, 9115.17],
   [9108.24, 9109.24, 9110.24, ..., 9113.24, 9114.24, 9115.24],
   ...,
   [9106.57, 9107.57, 9108.57, ..., 9111.57, 9112.57, 9113.57],
   [9106.37, 9107.37, 9108.37, ..., 9111.37, 9112.37, 9113.37],
   [9107.05, 9108.05, 9109.05, ..., 9112.05, 9113.05, 9114.05]]],


 [[[1005.36, 1006.36, 1007.36, ..., 1010.36, 1011.36, 1012.36],
   [1006.20, 1007.20, 1008.20, ..., 1011.20, 1012.20, 1013.20],
   [1006.27, 1007.27, 1008.27, ..., 1011.27, 1012.27, 1013.27],
   ...,
   [1004.60, 1005.60, 1006.60, ..., 1009.60, 1010.60, 1011.60],
   [1004.40, 1005.40, 1006.40, ..., 1009.40, 1010.40, 1011.40],
   [1005.08, 1006.08, 1007.08, ..., 1010.08, 1011.08, 1012.08]],

  [[1007.08, 1008.08, 1009.08, ..., 1012.08, 1013.08, 1014.08],
   [1007.92, 1008.92, 1009.92, ..., 1012.92, 1013.92, 1014.92],
   [1007.99, 1008.99, 1009.99, ..., 1012.99, 1013.99, 1014.99],
   ...,
   [1006.32, 1007.32, 1008.32, ..., 1011.32, 1012.32, 1013.32],
   [1006.12, 1007.12, 1008.12, ..., 1011.12, 1012.12, 1013.12],
   [1006.80, 1007.80, 1008.80, ..., 1011.80, 1012.80, 1013.80]],

  [[1011.75, 1012.75, 1013.75, ..., 1016.75, 1017.75, 1018.75],
   [1012.59, 1013.59, 1014.59, ..., 1017.59, 1018.59, 1019.59],
   [1012.66, 1013.66, 1014.66, ..., 1017.66, 1018.66, 1019.66],
   ...,
   [1010.99, 1011.99, 1012.99, ..., 1015.99, 1016.99, 1017.99],
   [1010.79, 1011.79, 1012.79, ..., 1015.79, 1016.79, 1017.79],
   [1011.47, 1012.47, 1013.47, ..., 1016.47, 1017.47, 1018.47]],

  ...,

  [[2100.99, 2101.99, 2102.99, ..., 2105.99, 2106.99, 2107.99],
   [2101.83, 2102.83, 2103.83, ..., 2106.83, 2107.83, 2108.83],
   [2101.90, 2102.90, 2103.90, ..., 2106.90, 2107.90, 2108.90],
   ...,
   [2100.24, 2101.24, 2102.24, ..., 2105.24, 2106.24, 2107.24],
   [2100.03, 2101.03, 2102.03, ..., 2105.03, 2106.03, 2107.03],
   [2100.71, 2101.71, 2102.71, ..., 2105.71, 2106.71, 2107.71]],

  [[3985.32, 3986.32, 3987.32, ..., 3990.32, 3991.32, 3992.32],
   [3986.16, 3987.16, 3988.16, ..., 3991.16, 3992.16, 3993.16],
   [3986.23, 3987.23, 3988.23, ..., 3991.23, 3992.23, 3993.23],
   ...,
   [3984.56, 3985.56, 3986.56, ..., 3989.56, 3990.56, 3991.56],
   [3984.36, 3985.36, 3986.36, ..., 3989.36, 3990.36, 3991.36],
   [3985.04, 3986.04, 3987.04, ..., 3990.04, 3991.04, 3992.04]],

  [[9107.44, 9108.44, 9109.44, ..., 9112.44, 9113.44, 9114.44],
   [9108.28, 9109.28, 9110.28, ..., 9113.28, 9114.28, 9115.28],
   [9108.35, 9109.35, 9110.35, ..., 9113.35, 9114.35, 9115.35],
   ...,
   [9106.69, 9107.69, 9108.69, ..., 9111.69, 9112.69, 9113.69],
   [9106.48, 9107.48, 9108.48, ..., 9111.48, 9112.48, 9113.48],
   [9107.16, 9108.16, 9109.16, ..., 9112.16, 9113.16, 9114.16]]]]";
        assert_eq!(format!("{:.2}", a), correct);
    }
}
