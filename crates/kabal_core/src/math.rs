use std::iter::Sum;

pub fn mean<'a, T: 'a>(numbers: &'a [T]) -> f32
where
    T: Sum<&'a T>,
    f32: From<T>,
{
    let sum: f32 = numbers.iter().sum::<T>().into();
    let length = numbers.len() as f32;

    sum / length
}
