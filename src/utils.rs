use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use jni::JNIEnv;
use jni_sys::{jclass, jint, jmethodID};

use crate::jvmti_libs::{jvmtiEnv, jvmtiError};

#[allow(unused)]
pub fn result_or_jvmti_err<T>(res: T, err_maybe: jvmtiError) -> Result<T, String> {
    if err_maybe as i32 != 0 {
        return Err(format!("Unexpected jvmti error: {:?}", err_maybe));
    }
    return Ok(res);
}

pub fn unit_or_jvmti_err(res: jvmtiError) -> Result<(), String> {
    if res as i32 != 0 {
        return Err(format!("Unexpected jvmti error: {:?}", res));
    }
    return Ok(());
}

#[allow(unused)]
pub unsafe fn find_method(
    jvmti_env: *mut jvmtiEnv,
    class: jclass,
    name: &str,
) -> Result<jmethodID, String> {
    // ref: http://stackoverflow.com/questions/42746496/call-class-method-from-manually-defined-class-in-jvmti
    let mut method_count: jint = 0;
    let mut methods: *mut jmethodID = ptr::null_mut();
    let meth_ret =
        (**jvmti_env).GetClassMethods.unwrap()(jvmti_env, class, &mut method_count, &mut methods);
    unit_or_jvmti_err(meth_ret)?;
    let method_slice: &[jmethodID] = slice::from_raw_parts_mut(methods, method_count as usize);
    let ret = method_slice.into_iter().find(|&&m| {
        let mut method_name: *mut c_char = ptr::null_mut();
        let name_ret = (**jvmti_env).GetMethodName.unwrap()(
            jvmti_env,
            m,
            &mut method_name,
            ptr::null_mut(),
            ptr::null_mut(),
        ) as i32;
        name_ret == 0 && CStr::from_ptr(method_name).to_str().unwrap() == name
    });
    return match ret {
        Some(&method) => Ok(method),
        None => Err("Method not found".to_string()),
    };
}

pub fn throw(message: String, env: &mut JNIEnv) {
    if let Err(err) = do_throw(message, env) {
        env.fatal_error(err.to_string());
    }
}

fn do_throw(message: String, env: &mut JNIEnv) -> jni::errors::Result<()> {
    let result = env.find_class("java/lang/Exception").unwrap();
    return env.throw_new(result, message.to_string());
}