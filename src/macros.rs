// A JNI call that is expected to return a non-null pointer when successful.
// If a null pointer is returned, it is converted to an Err.
// Returns Err if there is a pending exception after the call.
macro_rules! jni_non_null_call {
    ( $jnienv:expr, $name:tt $(, $args:expr )* ) => ({
        let res = jni_non_void_call!($jnienv, $name $(, $args)*);
        non_null!(res, concat!(stringify!($name), " result"))
    })
}

// A non-void JNI call. May return anything — primitives, references, error codes.
// Returns Err if there is a pending exception after the call.
macro_rules! jni_non_void_call {
    ( $jnienv:expr, $name:tt $(, $args:expr )* ) => ({
        println!("calling checked jni method: {}", stringify!($name));

        #[allow(unused_unsafe)]
        let res = unsafe {
            jni_method!($jnienv, $name)($jnienv, $($args),*)
        };

        check_exception!($jnienv);
        res
    })
}

macro_rules! non_null {
    ( $obj:expr, $ctx:expr ) => {
        if $obj.is_null() {
            return Err(String::from("obj is null"));
        } else {
            $obj
        }
    };
}

// A void JNI call.
// Returns Err if there is a pending exception after the call.
macro_rules! jni_void_call {
    ( $jnienv:expr, $name:tt $(, $args:expr )* ) => ({
        println!("calling checked jni method: {}", stringify!($name));

        #[allow(unused_unsafe)]
        unsafe {
            jni_method!($jnienv, $name)($jnienv, $($args),*)
        };

        check_exception!($jnienv);
    })
}

// A JNI call that does not check for exceptions or verify
// error codes (if any).
macro_rules! jni_unchecked {
    ( $jnienv:expr, $name:tt $(, $args:expr )* ) => ({
        println!("calling unchecked jni method: {}", stringify!($name));

        #[allow(unused_unsafe)]
        unsafe {
            jni_method!($jnienv, $name)($jnienv, $($args),*)
        }
    })
}

macro_rules! jni_method {
    ( $jnienv:expr, $name:tt ) => {{
        println!("looking up jni method {}", stringify!($name));
        let env = $jnienv;
        match deref!(deref!(env, "JNIEnv"), "*JNIEnv").$name {
            Some(method) => {
                println!("found jni method");
                method
            }
            None => {
                println!("jnienv method not defined, returning error");
                return Err(String::from("jnienv method not defined"));
            }
        }
    }};
}

macro_rules! check_exception {
    ( $jnienv:expr ) => {
        println!("checking for exception");
        let check = { jni_unchecked!($jnienv, ExceptionCheck) } == $crate::sys::JNI_TRUE;
        if check {
            println!("exception found, returning error");
            return Err(String::from("exception found, returning error"));
        }
        println!("no exception found");
    };
}

macro_rules! java_vm_unchecked {
    ( $java_vm:expr, $name:tt $(, $args:expr )* ) => ({
        println!("calling unchecked JavaVM method: {}", stringify!($name));
        java_vm_method!($java_vm, $name)($java_vm, $($args),*)
    })
}

macro_rules! java_vm_method {
    ( $jnienv:expr, $name:tt ) => {{
        println!("looking up JavaVM method {}", stringify!($name));
        let env = $jnienv;
        match deref!(deref!(env, "JavaVM"), "*JavaVM").$name {
            Some(meth) => {
                println!("found JavaVM method");
                meth
            }
            None => {
                println!("JavaVM method not defined, returning error");
                return Err(String::from("JavaVM method not defined"));
            }
        }
    }};
}

macro_rules! deref {
    ( $obj:expr, $ctx:expr ) => {
        if $obj.is_null() {
            return Err(String::from("ojb is null"));
        } else {
            #[allow(unused_unsafe)]
            unsafe {
                *$obj
            }
        }
    };
}

macro_rules! check_err {
    ( $obj:expr, $env: expr) => {{
        let return_res = $obj;
        if let Err(err) = return_res {
            throw(err.to_string(), $env);
            return JObject::null().as_raw();
        }else{
            return_res.unwrap()
        }
    }};
}