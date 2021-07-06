mod shader;

#[cfg(test)]
mod tests {
    use gl::types::*;

    macro_rules! gl_mock {
        (
            $struct_name:ident,
            [
                $($fn_name:ident($($arg_name:ident : $arg_typ:ty),*)),*
            ],
            [
                $($ret_fn_name:ident($($ret_arg_name:ident: $ret_arg_typ:ty),*) -> $ret_typ:ty),*
            ]
        ) => {
            $(#[allow(non_snake_case, dead_code)]
            fn $fn_name ($($arg_name: $arg_typ),*) {
                eprintln!(concat!(stringify!($fn_name), ": {:?}"), ($($arg_name),*));
            })*

            $(#[allow(non_snake_case, dead_code)]
            fn $ret_fn_name ($($ret_arg_name: $ret_arg_typ),*) -> $ret_typ {
                let ret = <$ret_typ>::default();
                eprintln!(concat!(stringify!($ret_fn_name), ": {:?} -> {:?}"), ($($ret_arg_name),*), ret);
                ret
            })*

            #[allow(non_snake_case)]
            pub(in crate::gl) struct $struct_name {
                $(pub(in crate::gl) $fn_name: fn($($arg_typ),*),)*
                $(pub(in crate::gl) $ret_fn_name: fn($($ret_arg_typ),*) -> $ret_typ,)*
            }

            impl Default for $struct_name {
                fn default() -> $struct_name {
                    $struct_name {
                        $($fn_name: $fn_name,)*
                        $($ret_fn_name: $ret_fn_name,)*
                     }
                }
            }

            impl $struct_name {
                pub(in crate::gl) fn get(&self, name: &str) -> *const () {
                    match name {
                        $(concat!("gl", stringify!($fn_name)) => self.$fn_name as *const _,)*
                        $(concat!("gl", stringify!($ret_fn_name)) => self.$ret_fn_name as *const _,)*
                        _ => std::ptr::null(),
                    }
                }

                pub(in crate::gl) fn load_all(&self) {
                    gl::load_with(|s| self.get(s) as *const _);
                }
            }
        }
    }

    gl_mock!(
        DummyGlLoader,
        [
            AttachShader(_program: GLuint, _shader: GLuint),
            CompileShader(_shader: GLuint),
            CreateShader(_type: GLenum),
            DeleteProgram(_program: GLuint),
            DeleteShader(_shader: GLuint),
            GetProgramInfoLog(
                _program: GLuint,
                _bufsize: GLint,
                _length: *mut i32,
                _info_log: *mut i8
            ),
            GetProgramiv(_program: GLuint, _pname: GLuint, params: *mut GLuint),
            GetShaderInfoLog(
                _shader: GLuint,
                _bufsize: GLint,
                _length: *mut i32,
                _info_log: *mut i8
            ),
            GetShaderiv(_shader: GLuint, _pname: GLuint, params: *mut GLuint),
            LinkProgram(_program: GLuint),
            ShaderSource(
                _shader: GLuint,
                _count: GLuint,
                _string: *const *const i8,
                _length: *const i32
            ),
            UseProgram(_program: GLuint)
        ],
        [CreateProgram() -> GLuint]
    );
}
