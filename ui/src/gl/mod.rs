mod elem_array;
mod shader;

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

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
                $(pub(in crate::gl) $fn_name: unsafe fn($($arg_typ),*),)*
                $(pub(in crate::gl) $ret_fn_name: unsafe fn($($ret_arg_typ),*) -> $ret_typ,)*
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
                pub(in crate::gl) fn get(&self, name: &str) -> *const c_void {
                    match name {
                        $(concat!("gl", stringify!($fn_name)) => self.$fn_name as *const _,)*
                        $(concat!("gl", stringify!($ret_fn_name)) => self.$ret_fn_name as *const _,)*
                        _ => std::ptr::null(),
                    }
                }

                pub(in crate::gl) fn load_all(&self) {
                    gl::load_with(|s| self.get(s));
                }
            }
        }
    }

    gl_mock!(
        DummyGlLoader,
        [
            AttachShader(_program: GLuint, _shader: GLuint),
            BindBuffer(_target: GLenum, _buffer: GLuint),
            BindVertexArray(_array: u32),
            BufferData(
                _target: GLenum,
                _size: isize,
                _data: *const c_void,
                _usage: GLenum
            ),
            BufferSubData(
                _target: GLenum,
                _offset: isize,
                _size: isize,
                _data: *const c_void
            ),
            CompileShader(_shader: GLuint),
            DeleteBuffers(_n: i32, _buffers: *const u32),
            DeleteVertexArrays(_n: i32, _buffers: *const u32),
            DeleteProgram(_program: GLuint),
            DeleteShader(_shader: GLuint),
            DrawElements(_mode: GLenum, _count: i32, _type: GLenum, _indices: *const c_void),
            GenBuffers(_n: i32, _buffers: *mut GLuint),
            GenVertexArrays(_n: i32, _arrays: *mut GLuint),
            GetProgramInfoLog(
                _program: GLuint,
                _bufsize: GLint,
                _length: *mut i32,
                _info_log: *mut i8
            ),
            GetProgramiv(_program: GLuint, _pname: GLuint, params: *mut GLint),
            GetShaderInfoLog(
                _shader: GLuint,
                _bufsize: GLint,
                _length: *mut i32,
                _info_log: *mut i8
            ),
            GetShaderiv(_shader: GLuint, _pname: GLuint, params: *mut GLint),
            LinkProgram(_program: GLuint),
            ShaderSource(
                _shader: GLuint,
                _count: GLint,
                _string: *const *const i8,
                _length: *const i32
            ),
            UseProgram(_program: GLuint),
            VertexAttribPointer(
                _index: u32,
                _size: GLint,
                _type: GLenum,
                _normalized: u8,
                _stride: i32,
                _pointer: *const c_void
            )
        ],
        [
            CreateProgram() -> GLuint,
            CreateShader(_type: GLenum) -> GLuint
        ]
    );
}
