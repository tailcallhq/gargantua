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
