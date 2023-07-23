use color_eyre::eyre::Result;

#[inline(always)]
pub fn set_func(
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let tmpl = v8::FunctionTemplate::new(scope, callback);
    let val = tmpl.get_function(scope).unwrap();
    val.set_name(key);
    obj.set(scope, key.into(), val.into());
}

pub trait OptionExt<T> {
    fn to_res(self, error_msg: &'static str) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn to_res(self, error_msg: &'static str) -> Result<T> {
        match self {
            Some(val) => Ok(val),
            None => return Err(color_eyre::eyre::eyre!(error_msg)),
        }
    }
}
