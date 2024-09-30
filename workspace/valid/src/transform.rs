use crate::{Valid, Validator};

pub trait Transform {
    type Value;
    type Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error>;

    fn pipe<Other>(self, other: Other) -> Pipe<Self, Other>
    where
        Self: Sized,
    {
        Pipe(self, other)
    }

    fn identity() -> Identity<Self::Value, Self::Error>
    where
        Self: Sized,
    {
        Identity(std::marker::PhantomData)
    }

    fn map_err<F, E>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E,
    {
        MapError(self, f)
    }
}

pub struct Pipe<A, B>(A, B);

impl<A: Transform, B> Transform for Pipe<A, B>
where
    B: Transform<Value = A::Value, Error = A::Error>,
{
    type Value = A::Value;
    type Error = A::Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        self.0
            .transform(input)
            .and_then(|input| self.1.transform(input))
    }
}

pub struct Identity<V, E>(std::marker::PhantomData<(V, E)>);

impl<V, E> Transform for Identity<V, E> {
    type Value = V;
    type Error = E;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::succeed(input)
    }
}

pub struct MapError<A, F>(A, F);

impl<A: Transform, F, E> Transform for MapError<A, F>
where
    F: Fn(A::Error) -> E,
{
    type Value = A::Value;
    type Error = E;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        self.0.transform(input).map_err(&self.1)
    }
}
