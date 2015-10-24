use std::ops::{Index, IndexMut};
use std::slice;
use std::vec;
use std::fmt;

use Dimension;

/// A multi-dimensional array.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Array<T> {
    dims: Vec<Dimension>,
    data: Vec<T>,
}

impl<T: fmt::Display> fmt::Display for Array<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.dims.iter().any(|dim| dim.lower_bound != 1) {
            for dim in &self.dims {
                try!(write!(fmt, "[{}:{}]", dim.lower_bound,
                            dim.lower_bound + dim.len as isize - 1));
            }
            try!(write!(fmt, "="));
        }
        fmt_helper(0, &self.dims, &mut self.data.iter(), fmt)
    }
}

fn fmt_helper<'a, T, I>(depth: usize,
                        dims: &[Dimension],
                        mut data: &mut I,
                        fmt: &mut fmt::Formatter)
                        -> fmt::Result
        where I: Iterator<Item=&'a T>, T: 'a + fmt::Display {
    if depth == dims.len() {
        return write!(fmt, "{}", data.next().unwrap());
    }

    try!(write!(fmt, "{{"));
    for i in 0..dims[depth].len {
        if i != 0 {
            try!(write!(fmt, ","));
        }
        try!(fmt_helper(depth + 1, dims, data, fmt));
    }
    write!(fmt, "}}")
}

impl<T> Array<T> {
    /// Creates a new `Array` from its underlying components.
    ///
    /// The data array should be provided in the higher-dimensional equivalent
    /// of row-major order.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements provided does not match the number of
    /// elements specified by the dimensions.
    pub fn from_parts(data: Vec<T>, dimensions: Vec<Dimension>) -> Array<T> {
        assert!((data.is_empty() && dimensions.is_empty()) ||
                data.len() == dimensions.iter().fold(1, |acc, i| acc * i.len),
                "size mismatch");
        Array {
            dims: dimensions,
            data: data,
        }
    }

    /// Creates a new one-dimensional array.
    pub fn from_vec(data: Vec<T>, lower_bound: isize) -> Array<T> {
        Array {
            dims: vec![Dimension {
                len: data.len(),
                lower_bound: lower_bound
            }],
            data: data,
        }
    }

    /// Wraps this array in a new dimension of size 1.
    ///
    /// For example, the one dimensional array `[1, 2]` would turn into the
    /// two-dimensional array `[[1, 2]]`.
    pub fn wrap(&mut self, lower_bound: isize) {
        self.dims.insert(0, Dimension {
            len: 1,
            lower_bound: lower_bound,
        });
    }

    /// Consumes another array, appending it to the top level dimension of this
    /// array.
    ///
    /// The dimensions of the other array must be the same as the dimensions
    /// of this array with the first dimension removed. This includes lower
    /// bounds as well as lengths.
    ///
    /// For example, if `[3, 4]` is pushed onto `[[1, 2]]`, the result is
    /// `[[1, 2], [3, 4]]`.
    ///
    /// # Panics
    ///
    /// Panics if the dimensions of the two arrays do not match.
    pub fn push(&mut self, other: Array<T>) {
        assert!(self.dims.len() - 1 == other.dims.len(),
                "cannot append differently shaped arrays");
        for (dim1, dim2) in self.dims.iter().skip(1).zip(other.dims.iter()) {
            assert!(dim1 == dim2, "cannot append differently shaped arrays");
        }
        self.dims[0].len += 1;
        self.data.extend(other.data);
    }

    /// Returns the dimensions of this array.
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dims
    }

    fn shift_idx(&self, indices: &[isize]) -> usize {
        assert_eq!(self.dims.len(), indices.len());
        self.dims
            .iter()
            .zip(indices.iter().cloned())
            .rev()
            .fold((0, 1), |(acc, stride), (dim, idx)| {
                let shifted = dim.shift(idx);
                (acc + shifted * stride, dim.len * stride)
            })
            .0
    }

    /// Returns an iterator over references to the elements of the array in the
    /// higher-dimensional equivalent of row-major order.
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            inner: self.data.iter(),
        }
    }

    /// Returns an iterator over mutable references to the elements of the
    /// array in the higher-dimensional equivalent of row-major order.
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut {
            inner: self.data.iter_mut(),
        }
    }
}

/// A trait implemented by types that can index into an `Array`.
pub trait ArrayIndex {
    /// Calculates the index into the `Array`'s underlying storage specified
    /// by the value of `self`.
    ///
    /// # Panics
    ///
    /// Panics if the value of `self` does not correspond to an in-bounds
    /// element of the `Array`.
    fn index<T>(&self, array: &Array<T>) -> usize;
}

impl<'a> ArrayIndex for &'a [isize] {
    fn index<T>(&self, array: &Array<T>) -> usize {
        array.shift_idx(*self)
    }
}

impl ArrayIndex for isize {
    fn index<T>(&self, array: &Array<T>) -> usize {
        let slice: &[isize] = &[*self];
        ArrayIndex::index(&slice, array)
    }
}

macro_rules! tuple_impl {
    ($($name:ident : $t:ty),+) => {
        impl ArrayIndex for ($($t,)+) {
            fn index<T>(&self, array: &Array<T>) -> usize {
                let ($($name,)+) = *self;
                let slice: &[isize] = &[$($name),+];
                ArrayIndex::index(&slice, array)
            }
        }
    }
}

tuple_impl!(a: isize);
tuple_impl!(a: isize, b: isize);
tuple_impl!(a: isize, b: isize, c: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize, e: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize, e: isize, f: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize, e: isize, f: isize, g: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize, e: isize, f: isize, g: isize, h: isize);
tuple_impl!(a: isize, b: isize, c: isize, d: isize, e: isize, f: isize, g: isize, h: isize, i: isize);

impl<T, I: ArrayIndex> Index<I> for Array<T> {
    type Output = T;

    /// Indexes into the `Array`, retrieving a reference to the contained
    /// value.
    ///
    /// Since `Array`s can be multi-dimensional, the `Index` trait is
    /// implemented for a variety of index types. In the most generic case, a
    /// `&[isize]` can be used. In addition, a bare `isize` as well as tuples
    /// of up to 10 `isize` values may be used for convenience.
    ///
    /// # Panics
    ///
    /// Panics if the index does not correspond to an in-bounds element of the
    /// `Array`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use postgres_array::Array;
    /// let mut array = Array::from_vec(vec![0i32, 1, 2, 3], 0);
    /// assert_eq!(2, array[2]);
    ///
    /// array.wrap(0);
    /// array.push(Array::from_vec(vec![4, 5, 6, 7], 0));
    ///
    /// assert_eq!(6, array[(1, 2)]);
    /// ```
    fn index(&self, idx: I) -> &T {
        let idx = idx.index(self);
        &self.data[idx]
    }
}

impl<T, I: ArrayIndex> IndexMut<I> for Array<T> {
    fn index_mut(&mut self, idx: I) -> &mut T {
        let idx = idx.index(self);
        &mut self.data[idx]
    }
}

impl<'a, T: 'a> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a mut Array<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self.data.into_iter()
        }
    }
}

/// An iterator over references to values of an `Array` in the
/// higher-dimensional equivalent of row-major order.
pub struct Iter<'a, T: 'a> {
    inner: slice::Iter<'a, T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.inner.next_back()
    }
}

/// An iterator over mutable references to values of an `Array` in the
/// higher-dimensional equivalent of row-major order.
pub struct IterMut<'a, T: 'a> {
    inner: slice::IterMut<'a, T>,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        self.inner.next()
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> {
        self.inner.next_back()
    }
}

/// An iterator over values of an `Array` in the higher-dimensional
/// equivalent of row-major order.
pub struct IntoIter<T> {
    inner: vec::IntoIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.next()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.inner.next_back()
    }
}
