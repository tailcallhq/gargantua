use valid::{Valid, Validator};

pub trait Transform: Sized {
    type Value;
    type Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error>;

    fn pipe<B: Transform>(self, other: B) -> Pipe<Self, B> {
        Pipe(self, other)
    }
}

pub struct Pipe<A, B>(A, B);
impl<A: Transform, B: Transform<Value = A::Value, Error = A::Error>> Transform for Pipe<A, B> {
    type Value = A::Value;
    type Error = A::Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        self.0.transform(input).and_then(|d| self.1.transform(d))
    }
}
