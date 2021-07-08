use gl::types::{GLenum, GLuint};

pub(in crate) struct ActiveShaderProgram<'a> {
    shader: &'a mut ShaderProgram,
}

pub(crate) struct ShaderProgram {
    program: GLuint,
}

impl ShaderProgram {
    pub(crate) fn new(
        vertex_shader_source: &str,
        fragment_shader_str: &str,
    ) -> Result<ShaderProgram, String> {
        let mut success = 1;
        let mut len = 0;
        let mut info_log = [0; 512];
        let vshdr = Shader::new(vertex_shader_source, gl::VERTEX_SHADER, "vertex")?;
        let fshdr = Shader::new(fragment_shader_str, gl::FRAGMENT_SHADER, "fragment")?;
        unsafe {
            let id = gl::CreateProgram();
            gl::AttachShader(id, vshdr.0);
            gl::AttachShader(id, fshdr.0);
            gl::LinkProgram(id);
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                gl::GetProgramInfoLog(id, 512, &mut len, info_log.as_mut_ptr() as *mut i8);
                let info_str = std::str::from_utf8(&info_log[..(len as usize)]).unwrap();
                Err(format!("failed to link shader program: {}", info_str))
            } else {
                Ok(ShaderProgram { program: id })
            }
        }
    }

    // Use shader program
    pub(crate) fn use_program<'a>(&'a mut self) -> ActiveShaderProgram<'a> {
        unsafe {
            gl::UseProgram(self.program);
        }
        ActiveShaderProgram { shader: self }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program) }
    }
}

// Handle to an individual shader compilation unit
struct Shader(GLuint);

impl Shader {
    // Compile shader from source
    fn new(src: &str, typ: GLenum, name: &str) -> Result<Shader, String> {
        let mut success = 1;
        let mut len = 0;
        let mut info_log = [0; 512];
        unsafe {
            let id = gl::CreateShader(typ);
            gl::ShaderSource(id, 1, &(src.as_ptr() as *const i8), &(src.len() as i32));
            gl::CompileShader(id);
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                gl::GetShaderInfoLog(id, 512, &mut len, info_log.as_mut_ptr() as *mut i8);
                let info_str = std::str::from_utf8(&info_log[..(len as usize)]).unwrap();
                Err(format!("failed to compile {} shader: {}", name, info_str))
            } else {
                Ok(Shader(id))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}
