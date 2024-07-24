use std::ffi::c_void;
use std::ptr::null_mut;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::*;

use crate::jvmti_libs::{
    JVMTI_VERSION, jvmtiCapabilities, jvmtiEnv,
    jvmtiHeapObjectFilter_JVMTI_HEAP_OBJECT_EITHER, jvmtiIterationControl,
    jvmtiIterationControl_JVMTI_ITERATION_ABORT, jvmtiIterationControl_JVMTI_ITERATION_CONTINUE,
};
use crate::utils::{throw, unit_or_jvmti_err};

#[macro_use]
mod macros;
#[allow(warnings)]
mod jvmti_libs;
mod utils;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_App_helloJNI(mut env: JNIEnv, _this: JClass) -> jstring {
    let local_env = &mut env;
    return local_env
        .new_string("SuccessTest")
        .unwrap_or_else(|e| {
            throw(e.to_string(), local_env);
            return JString::from(JObject::null());
        })
        .into_raw();
}

#[derive(Debug)]
pub struct CallbackParam {
    instance_count: i32,
    max_instances: i32,
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_App_getInstances<'local>(
    mut env: JNIEnv,
    _this: JClass<'local>,
    target_clazz: JClass<'local>,
    limit_num: jint,
) -> jobjectArray {
    let local_env = &mut env;

    let vm = check_err!(local_env.get_java_vm(), local_env).get_java_vm_pointer();

    let jvmti_env = check_err!(get_jvmti_env(vm), local_env);

    check_err!(add_capabilities(jvmti_env), local_env);

    check_err!(
        iterate_over_instances_of_class(jvmti_env, target_clazz.as_raw(), limit_num),
        local_env
    );

    let tags = check_err!(get_objects_with_tags(local_env, jvmti_env), local_env);

    let object_array = check_err!(
        local_env.new_object_array(tags.0, target_clazz, JObject::null()),
        local_env
    );

    for i in 0..tags.0 {
        check_err!(
            local_env.set_object_array_element(
                &object_array,
                i,
                JObject::from_raw(*tags.1.offset(i as isize)),
            ),
            local_env
        );
    }
    check_err!(
        unit_or_jvmti_err((**jvmti_env).Deallocate.unwrap()(jvmti_env, tags.1.cast())),
        local_env
    );

    return object_array.as_raw();
}

#[allow(unused)]
pub unsafe fn get_jvmti_env(vm: *mut JavaVM) -> Result<*mut jvmtiEnv, String> {
    let mut ptr = null_mut();
    java_vm_unchecked!(vm, GetEnv, &mut ptr, JVMTI_VERSION);
    Ok(ptr as *mut jvmtiEnv)
}

#[allow(unused)]
pub unsafe fn get_jvmti_env_(vm: *mut JavaVM) -> Result<*mut jvmtiEnv, String> {
    let mut ptr: *mut c_void = null_mut();
    let env_res = (**vm).GetEnv.unwrap()(vm, &mut ptr, JVMTI_VERSION);
    if env_res != JNI_OK {
        return Err(format!("No environment, err: {}", env_res));
    }
    return Ok(ptr as *mut jvmtiEnv);
}

pub unsafe fn add_capabilities(jvmti_env: *mut jvmtiEnv) -> Result<(), String> {
    let mut caps = jvmtiCapabilities::default();
    caps.set_can_tag_objects(1);
    return unit_or_jvmti_err((**jvmti_env).AddCapabilities.unwrap()(jvmti_env, &caps));
}

pub unsafe fn iterate_over_instances_of_class(
    jvmti_env: *mut jvmtiEnv,
    target_clazz: jclass,
    limit_num: jint,
) -> Result<(), String> {
    let mut user_data = CallbackParam {
        instance_count: 0,
        max_instances: limit_num,
    };
    return unit_or_jvmti_err((**jvmti_env).IterateOverInstancesOfClass.unwrap()(
        jvmti_env,
        target_clazz,
        jvmtiHeapObjectFilter_JVMTI_HEAP_OBJECT_EITHER,
        Some(jvmti_heap_object_callback),
        &mut user_data as *mut CallbackParam as *mut c_void,
    ));
}

pub unsafe extern "C" fn jvmti_heap_object_callback(
    _class_tag: jlong,
    _size: jlong,
    tag_ptr: *mut jlong,
    user_data: *mut c_void,
) -> jvmtiIterationControl {
    let data = &mut *(user_data as *mut CallbackParam);
    if data.max_instances <= 0 {
        return jvmtiIterationControl_JVMTI_ITERATION_ABORT;
    }
    data.instance_count += 1;
    *tag_ptr = 1;
    // 检查是否达到了限定数量
    if data.instance_count >= data.max_instances {
        return jvmtiIterationControl_JVMTI_ITERATION_ABORT; // 停止迭代
    }
    return jvmtiIterationControl_JVMTI_ITERATION_CONTINUE;
}

pub unsafe fn get_objects_with_tags<'local>(
    env: &mut JNIEnv,
    jvmti_env: *mut jvmtiEnv,
) -> Result<(jint, *mut jobject), String> {
    let mut count: jint = 0;
    let mut objects: *mut jobject = null_mut();
    if let Err(err) = unit_or_jvmti_err((**jvmti_env).GetObjectsWithTags.unwrap()(
        jvmti_env,
        1,
        &1,
        &mut count,
        &mut objects,
        null_mut(),
    )) {
        throw(err.clone(), env);
        return Err(err);
    }
    return Ok((count, objects));
}