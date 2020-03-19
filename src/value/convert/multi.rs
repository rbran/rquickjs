use super::{Args, FromJs, FromJsMulti, ToJs, ToJsMulti};
use crate::{Ctx, Error, Result, Value};

impl<'js> ToJsMulti<'js> for Vec<Value<'js>> {
    fn to_js_multi(self, _: Ctx<'js>) -> Result<Vec<Value<'js>>> {
        Ok(self)
    }
}

impl<'js, T: ToJs<'js>> ToJsMulti<'js> for T {
    fn to_js_multi(self, ctx: Ctx<'js>) -> Result<Vec<Value<'js>>> {
        Ok(vec![self.to_js(ctx)?])
    }
}

impl<'js> FromJsMulti<'js> for Args<'js> {
    fn from_js_multi(_: Ctx<'js>, value: Vec<Value<'js>>) -> Result<Self> {
        Ok(Args(value))
    }
}

impl<'js, T: FromJs<'js>> FromJsMulti<'js> for T {
    fn from_js_multi(ctx: Ctx<'js>, value: Vec<Value<'js>>) -> Result<Self> {
        let len = value.len();
        let v = value
            .into_iter()
            .next()
            .ok_or(Error::MissingArguments(len, 1))?;
        T::from_js(ctx, v)
    }
}

macro_rules! impl_to_js_multi{
    ($($t:ident),+) => {
        impl<'js, $($t,)*> ToJsMulti<'js> for ($($t,)*)
            where $($t: ToJs<'js>,)*
        {
            #[allow(non_snake_case)]
            fn to_js_multi(self, ctx: Ctx<'js>) -> Result<Vec<Value<'js>>>{
                let ($($t,)*) = self;
                Ok(vec![
                    $($t.to_js(ctx)?,)*
                ])
            }
        }
    }
}

macro_rules! impl_from_js_multi{
    ($num:expr, $($t:ident),+) => {
        impl<'js, $($t,)*> FromJsMulti<'js> for ($($t,)*)
            where $($t: FromJs<'js>,)*
        {
            #[allow(non_snake_case)]
            fn from_js_multi(ctx: Ctx<'js>, value: Vec<Value<'js>>) -> Result<Self> {
                let len = value.len();
                let mut iter = value.into_iter();
                Ok((
                    $({
                        let v = iter.next()
                            .ok_or(Error::MissingArguments(len,1))?;
                        $t::from_js(ctx,v)?
                    },)*
                ))
            }
        }
    }
}

impl_to_js_multi!(A);
impl_to_js_multi!(A, B);
impl_to_js_multi!(A, B, C);
impl_to_js_multi!(A, B, C, D);
impl_to_js_multi!(A, B, C, D, E);
impl_to_js_multi!(A, B, C, D, E, F);
impl_to_js_multi!(A, B, C, D, E, F, G);
impl_to_js_multi!(A, B, C, D, E, F, G, H);
impl_to_js_multi!(A, B, C, D, E, F, G, H, I);
impl_to_js_multi!(A, B, C, D, E, F, G, H, I, J);
impl_to_js_multi!(A, B, C, D, E, F, G, H, I, J, K);

impl_from_js_multi!(1, A);
impl_from_js_multi!(2, A, B);
impl_from_js_multi!(3, A, B, C);
impl_from_js_multi!(4, A, B, C, D);
impl_from_js_multi!(5, A, B, C, D, E);
impl_from_js_multi!(6, A, B, C, D, E, F);
impl_from_js_multi!(7, A, B, C, D, E, F, G);
impl_from_js_multi!(8, A, B, C, D, E, F, G, H);
impl_from_js_multi!(9, A, B, C, D, E, F, G, H, I);
impl_from_js_multi!(10, A, B, C, D, E, F, G, H, I, J);
impl_from_js_multi!(11, A, B, C, D, E, F, G, H, I, J, K);