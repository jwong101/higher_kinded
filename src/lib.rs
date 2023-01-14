use std::marker::PhantomData;

trait K1 {
    type Inner;
    // The bound `With<I>: K1<Inner = I>` ensures that Inner matches the last applied type parameter.
    // The bound `K1<With<Self::Inner> = Self>` ensures that the projection's projection points
    // back to `Self`. This is to disallow impl's of the form:
    // ```rust
    // struct Id1<T>(T);
    // struct Id2<T>(T);
    // impl<T> K1 for Id1<T> {
    //     type Inner = T;
    //     type With<I> = Id2<I>;
    // }
    // impl<T> K1 for Id2<T> {
    //     type Inner = T;
    //     type With<I> = Id2<I>;
    // }
    // ```
    //
    // We need the third bound to ensure that the projection's projection is the same as the
    // projection of `Self`. Note that if this bound was not present, then the following impl would
    // be legal:
    // ```rust
    // struct Id1<T>(T);
    // struct Id2<T>(T);
    // impl<T> K1 for Id1<T> {
    //     type Inner = T;
    //     type With<I> = Id2<I>;
    // }
    // impl<T> K1 for Id2<T> {
    //     type Inner = T;
    //     type With<I> = Id1<I>;
    // }
    // ```
    type With<I>: K1<Inner = I> + K1<With<Self::Inner> = Self> + K1<With<I> = Self::With<I>>;
}

trait Functor: K1 {
    fn fmap<B>(self, f: impl FnOnce(Self::Inner) -> B) -> Self::With<B>;
}

trait Applicative: Functor {
    fn pure(val: Self::Inner) -> Self;

    fn zip_with<B, C>(self, b: Self::With<B>, f: impl FnOnce(Self::Inner, B) -> C)
        -> Self::With<C>;
}

trait Monad: Applicative {
    fn bind<B>(self, f: impl FnOnce(Self::Inner) -> Self::With<B>) -> Self::With<B>;

    fn flatten(self) -> Self::Inner
    where
        Self::Inner: K1<With<Self::Inner> = Self>;
}

#[derive(Debug, PartialEq, Eq)]
struct Identity<T>(T);

impl<T> K1 for Identity<T> {
    type Inner = T;

    type With<I> = Identity<I>;
}

impl<A> Functor for Identity<A> {
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Identity<B> {
        Identity(f(self.0))
    }
}

impl<A> Applicative for Identity<A> {
    fn pure(val: A) -> Identity<A> {
        Identity(val)
    }

    fn zip_with<B, C>(self, b: Identity<B>, f: impl FnOnce(A, B) -> C) -> Identity<C> {
        Identity(f(self.0, b.0))
    }
}

impl<A> Monad for Identity<A> {
    fn bind<B>(self, f: impl FnOnce(A) -> Identity<B>) -> Identity<B> {
        f(self.0)
    }

    fn flatten(self) -> Self::Inner
    where
        Self::Inner: K1<With<A> = Self>,
    {
        self.0
    }
}

struct Const<C, V> {
    inner: C,
    _marker: PhantomData<V>,
}

impl<C, V> K1 for Const<C, V> {
    type Inner = V;

    type With<I> = Const<C, I>;
}

impl<C, A> Functor for Const<C, A> {
    fn fmap<B>(self, _: impl FnOnce(A) -> B) -> Const<C, B> {
        // mfw no type-changing-struct-update
        Const {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident() {
        let i = Identity(Identity(0));
        assert_eq!(i.flatten(), Identity(0));
    }
}
