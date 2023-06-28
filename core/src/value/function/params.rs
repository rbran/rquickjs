use crate::{
    function::{Exhaustive, Flat, Func, Opt, Rest, This},
    qjs, Ctx, FromJs, Result, Value,
};
use std::slice;

/// A struct which contains the values a callback is called with.
pub struct Params<'a, 'js> {
    ctx: Ctx<'js>,
    function: qjs::JSValue,
    this: qjs::JSValue,
    args: &'a [qjs::JSValue],
}

impl<'a, 'js> Params<'a, 'js> {
    pub(crate) unsafe fn from_ffi_class(
        ctx: *mut qjs::JSContext,
        function: qjs::JSValue,
        this: qjs::JSValue,
        argc: qjs::c_int,
        argv: *mut qjs::JSValue,
        _flags: qjs::c_int,
    ) -> Self {
        let argc = usize::try_from(argc).expect("invalid argument number");
        let args = slice::from_raw_parts(argv, argc);
        Self {
            ctx: Ctx::from_ptr(ctx),
            function,
            this,
            args,
        }
    }

    pub(crate) unsafe fn from_ffi_c_func(
        ctx: *mut qjs::JSContext,
        this: qjs::JSValue,
        argc: qjs::c_int,
        argv: *mut qjs::JSValue,
    ) -> Self {
        let argc = usize::try_from(argc).expect("invalid argument number");
        let args = slice::from_raw_parts(argv, argc);
        Self {
            ctx: Ctx::from_ptr(ctx),
            function: qjs::JS_UNDEFINED,
            this,
            args,
        }
    }

    /// Checks if the parameters fit the param num requirements.
    pub fn check_params(&self, num: ParamReq) -> Result<()> {
        if self.args.len() < num.min {
            return Err(crate::Error::MissingArgs {
                expected: num.min,
                given: self.args.len(),
            });
        }
        if num.exhaustive && self.args.len() > num.max {
            return Err(crate::Error::TooManyArgs {
                expected: num.max,
                given: self.args.len(),
            });
        }
        Ok(())
    }

    /// Returns the context assiociated with call.
    pub fn ctx(&self) -> Ctx<'js> {
        self.ctx
    }

    /// Returns the value on which this function called. i.e. in `bla.foo()` the `foo` value.
    pub fn function(&self) -> Value<'js> {
        unsafe { Value::from_js_value_const(self.ctx, self.function) }
    }

    /// Returns the this on which this function called. i.e. in `bla.foo()` the `bla` value.
    pub fn this(&self) -> Value<'js> {
        unsafe { Value::from_js_value_const(self.ctx, self.function) }
    }

    /// Returns the argument at a given index..
    pub fn arg(&self, index: usize) -> Option<Value<'js>> {
        self.args
            .get(index)
            .map(|arg| unsafe { Value::from_js_value_const(self.ctx, *arg) })
    }

    /// Returns the number of arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Returns if there are no arguments
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Turns the params into an accessor object for extracting the arguments.
    pub fn access(self) -> ParamsAccessor<'a, 'js> {
        ParamsAccessor {
            params: self,
            offset: 0,
        }
    }
}

pub struct ParamsAccessor<'a, 'js> {
    params: Params<'a, 'js>,
    offset: usize,
}

impl<'a, 'js> ParamsAccessor<'a, 'js> {
    /// Returns the context associated with the params.
    pub fn ctx(&self) -> Ctx<'js> {
        self.params.ctx()
    }

    /// Returns this value of call from which the params originate.
    pub fn this(&self) -> Value<'js> {
        self.params.this()
    }

    /// Returns the value on which this function called. i.e. in `bla.foo()` the `foo` value.
    pub fn function(&self) -> Value<'js> {
        self.params.function()
    }

    /// Returns the next arguments.
    ///
    /// Each call to this function returns a different argument
    ///
    /// # Panic
    /// This function panics if it is called more times then there are arguments.
    pub fn arg(&mut self) -> Value<'js> {
        assert!(
            self.offset < self.params.args.len(),
            "arg called too many times"
        );
        let res = self.params.args[self.offset];
        self.offset += 1;
        // TODO: figure out ownership
        unsafe { Value::from_js_value(self.params.ctx, res) }
    }

    /// returns the number of arguments remaining
    pub fn len(&self) -> usize {
        self.params.args.len() - self.offset
    }
    /// returns whether there are any arguments remaining.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A struct encoding the requirements of a parameter set.
pub struct ParamReq {
    min: usize,
    max: usize,
    exhaustive: bool,
}

impl ParamReq {
    /// Returns the requirement of a single required parameter
    pub const fn single() -> Self {
        ParamReq {
            min: 1,
            max: 1,
            exhaustive: false,
        }
    }

    /// Makes the requirements exhaustive i.e. the parameter set requires that the function is
    /// called with no arguments than parameters
    pub const fn exhaustive() -> Self {
        ParamReq {
            min: 0,
            max: 0,
            exhaustive: true,
        }
    }

    /// Returns the requirements for a single optional parameter
    pub const fn optional() -> Self {
        ParamReq {
            min: 0,
            max: 1,
            exhaustive: false,
        }
    }

    /// Returns the requirements for a any number of parameters
    pub const fn any() -> Self {
        ParamReq {
            min: 0,
            max: usize::MAX,
            exhaustive: false,
        }
    }

    /// Returns the requirements for no parameters
    pub const fn none() -> Self {
        ParamReq {
            min: 0,
            max: 0,
            exhaustive: false,
        }
    }

    /// Combine to requirements into one which covers both.
    pub const fn combine(self, other: Self) -> ParamReq {
        Self {
            min: self.min.saturating_add(other.min),
            max: self.max.saturating_add(other.max),
            exhaustive: self.exhaustive || other.exhaustive,
        }
    }

    /// Returns the minimum number of arguments for this requirement
    pub fn min(&self) -> usize {
        self.min
    }

    /// Returns the maximum number of arguments for this requirement
    pub fn max(&self) -> usize {
        self.max
    }

    /// Returns whether this function is required to be exhaustive called
    ///
    /// i.e. there can be no more arguments then parameters.
    pub fn is_exhaustive(&self) -> bool {
        self.exhaustive
    }
}

/// A trait to extract argument values.
pub trait FromParam<'js>: Sized {
    /// The parameters requirements this value requires.
    fn params_required() -> ParamReq;

    /// Convert from a parameter value.
    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self>;
}

impl<'js, T: FromJs<'js>> FromParam<'js> for T {
    fn params_required() -> ParamReq {
        ParamReq::single()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        T::from_js(params.ctx(), params.arg())
    }
}

impl<'js, T: FromJs<'js>> FromParam<'js> for Opt<T> {
    fn params_required() -> ParamReq {
        ParamReq::optional()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        if !params.is_empty() {
            Ok(Opt(Some(T::from_js(params.ctx(), params.arg())?)))
        } else {
            Ok(Opt(None))
        }
    }
}

impl<'js, T: FromJs<'js>> FromParam<'js> for This<T> {
    fn params_required() -> ParamReq {
        ParamReq::any()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        T::from_js(params.ctx(), params.this()).map(This)
    }
}

impl<'js, T: FromJs<'js>> FromParam<'js> for Func<T> {
    fn params_required() -> ParamReq {
        ParamReq::any()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        T::from_js(params.ctx(), params.function()).map(Func)
    }
}

impl<'js, T: FromJs<'js>> FromParam<'js> for Rest<T> {
    fn params_required() -> ParamReq {
        ParamReq::any()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let mut res = Vec::with_capacity(params.len());
        for _ in 0..params.len() {
            let p = params.arg();
            res.push(T::from_js(params.ctx(), p)?);
        }
        Ok(Rest(res))
    }
}

impl<'js, T: FromParams<'js>> FromParam<'js> for Flat<T> {
    fn params_required() -> ParamReq {
        T::params_requirements()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        T::from_params(params).map(Flat)
    }
}

impl<'js> FromParam<'js> for Exhaustive {
    fn params_required() -> ParamReq {
        ParamReq::exhaustive()
    }

    fn from_param<'a>(_params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Exhaustive)
    }
}

/// A trait to extract a tuple of argument values.
pub trait FromParams<'js>: Sized {
    /// The parameters requirements this value requires.
    fn params_requirements() -> ParamReq;

    /// Convert from a parameter value.
    fn from_params<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self>;
}

macro_rules! impl_from_params{
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<'js $(,$t)*> FromParams<'js> for ($($t,)*)
        where
            $($t : FromParam<'js>,)*
        {
            fn params_requirements() -> ParamReq{
                ParamReq::none()
                    $(.combine($t::params_required()))*
            }

            fn from_params<'a>(_args: &mut ParamsAccessor<'a,'js>) -> Result<Self>{
                Ok((
                    $($t::from_param(_args)?,)*
                ))
            }
        }
    };
}

impl_from_params!();
impl_from_params!(A);
impl_from_params!(A, B);
impl_from_params!(A, B, C);
impl_from_params!(A, B, C, D);
impl_from_params!(A, B, C, D, E);
impl_from_params!(A, B, C, D, E, F);
impl_from_params!(A, B, C, D, E, F, G);