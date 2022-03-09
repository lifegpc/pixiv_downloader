use std::clone::Clone;
use std::cmp::PartialEq;
use std::convert::AsMut;
use std::convert::AsRef;
use std::convert::From;
use std::convert::Into;
use std::default::Default;
use std::fmt::Debug;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::FromStr;

/// A list which if index greater than count, which will return last one of the element.
pub struct NonTailList<T> {
    /// The list
    data: Vec<T>,
}

impl<T> AddAssign<T> for NonTailList<T> {
    fn add_assign(&mut self, rhs: T) {
        self.data.push(rhs)
    }
}

impl<T: Clone> AddAssign<&T> for NonTailList<T> {
    fn add_assign(&mut self, rhs: &T) {
        self.data.push(rhs.clone())
    }
}

impl<T> AsMut<NonTailList<T>> for NonTailList<T> {
    fn as_mut(&mut self) -> &mut NonTailList<T> {
        self
    }
}

impl<T> AsRef<NonTailList<T>> for NonTailList<T> {
    fn as_ref(&self) -> &NonTailList<T> {
        self
    }
}

impl<T: Clone> Clone for NonTailList<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone()
        }
    }
}

impl<T: Debug> Debug for NonTailList<T> where Vec<T>: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Vec::<T>::fmt(&self.data, f)
    }
}

impl<T> Default for NonTailList<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
        }
    }
}

impl<T: Clone> From<&[T]> for NonTailList<T> {
    fn from(list: &[T]) -> Self {
        Self {
            data: Vec::from(list),
        }
    }
}

impl<T> From<Vec<T>> for NonTailList<T> {
    fn from(list: Vec<T>) -> Self {
        Self {
            data: list,
        }
    }
}

impl<T: Clone> From<&Vec<T>> for NonTailList<T> {
    fn from(list: &Vec<T>) -> Self {
        Self {
            data: list.clone(),
        }
    }
}

impl<T: FromStr> FromStr for NonTailList<T> {
    type Err = T::Err;
    /// Parse a list from string
    /// * `s` - a list of items separated by `,`
    /// # Examples
    /// ```
    /// let l: NonTailList<i32> = NonTailList::from_str("1,2,3").unwrap();
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        let mut l: Vec<T> = Vec::new();
        let r = s.split(",");
        for i in r {
            let i = i.trim();
            let a = i.parse::<T>()?;
            l.push(a);
        }
        Ok(Self {
            data: l,
        })
    }
}

impl<T> Into<Vec<T>> for NonTailList<T> {
    fn into(self) -> Vec<T> {
        self.data
    }
}

impl<T: Clone> Into<Vec<T>> for &NonTailList<T> {
    fn into(self) -> Vec<T> {
        self.data.clone()
    }
}

impl<T> Index<usize> for NonTailList<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let count = self.data.len();
        if index < count {
            &self.data[index]
        } else {
            &self.data[count - 1]
        }
    }
}

impl<T> IndexMut<usize> for NonTailList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let count = self.data.len();
        if index < count {
            self.data.index_mut(index)
        } else {
            self.data.index_mut(count - 1)
        }
    }
}

impl<T, V> PartialEq<Vec<V>> for NonTailList<T> where T: PartialEq<V> {
    fn eq(&self, other: &Vec<V>) -> bool {
        &self.data == other
    }
}

#[test]
fn test_non_tail_list() {
    let mut l = NonTailList::from(vec![1, 2, 3]);
    assert_eq!(1, l[0]);
    assert_eq!(2, l[1]);
    assert_eq!(3, l[2]);
    assert_eq!(3, l[3]);
    l[0] = 0;
    assert_eq!(0, l[0]);
    let l2: Vec<i32> = l.as_ref().into();
    assert_eq!(l2, vec![0, 2, 3]);
    assert_eq!(l[4], 3);
    l += 4;
    assert_eq!(l[3], 4);
    let l = NonTailList::<i32>::from_str("1, 2, 3").unwrap();
    assert!(l == vec![1, 2, 3]);
    assert_eq!(l, vec![1, 2, 3]);
    let l2: Vec<i32> = l.as_ref().into();
    assert_eq!(l2, vec![1, 2, 3]);
}
