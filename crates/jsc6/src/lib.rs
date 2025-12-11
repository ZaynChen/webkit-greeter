// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

use jsc6::{
    ffi::{jsc_value_new_function_variadic, jsc_value_new_object, JSCClass, JSCValue},
    glib::{
        ffi::{gpointer, GPtrArray},
        object::IsA,
        translate::*,
        types::StaticType,
    },
};

use std::ptr::null_mut;

pub use jsc6::*;

pub trait JSCValueExtManual: IsA<Value> + 'static {
    #[doc(alias = "jsc_value_new_object")]
    fn new_object(context: &Context, instance: Option<Value>, jsc_class: Option<&Class>) -> Value {
        let jsc_class = match jsc_class {
            Some(cls) => cls.to_glib_none().0,
            None => null_mut::<JSCClass>(),
        };

        let instance = match instance {
            Some(ins) => ins.to_glib_none().0,
            None => null_mut::<JSCValue>(),
        };

        unsafe {
            from_glib_none(jsc_value_new_object(
                context.to_glib_none().0,
                instance as gpointer,
                jsc_class,
            ))
        }
    }

    #[doc(alias = "jsc_value_new_function_variadic")]
    fn new_function_variadic<F>(context: &Context, name: Option<&str>, callback: F) -> Value
    where
        F: Fn(&[Value]) -> Option<Value> + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            params: *mut GPtrArray,
            user_data: gpointer,
        ) -> Option<Value>
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            unsafe {
                let f: &F = &*(user_data as *const F);
                f(&Value::from_glib_none_as_vec(params))
            }
        }

        unsafe extern "C" fn destroy_closure<F>(user_data: gpointer)
        where
            F: Fn(&[Value]) -> Option<Value>,
        {
            // destroy
            unsafe {
                let _ = Box::<F>::from_raw(user_data as *mut _);
            }
        }

        unsafe {
            let callback = Box::into_raw(Box::new(callback));
            from_glib_none(jsc_value_new_function_variadic(
                context.to_glib_none().0,
                name.to_glib_none().0,
                Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(
                    trampoline::<F> as *const (),
                )),
                callback as gpointer,
                Some(destroy_closure::<F>),
                Value::static_type().into_glib(),
            ))
        }
    }

    fn to_vec(&self) -> Vec<Value> {
        let this = self.as_ref();
        if !this.is_array() {
            panic!("JSCValue is not an array");
        }

        let length = this
            .object_get_property("length")
            .expect("object does not has property `length`")
            .to_int32() as u32;
        let mut array = Vec::with_capacity(length as usize);
        for i in 0..length {
            array.push(this.object_get_property_at_index(i).unwrap());
        }

        array
    }
}
impl<O: IsA<Value>> JSCValueExtManual for O {}
