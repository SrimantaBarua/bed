use std::marker::PhantomData;

use gl::types::GLuint;

use super::shader::ActiveShaderProgram;

pub(crate) struct VertexAttrs {
    pub(crate) size: i32,
    pub(crate) stride: usize,
    pub(crate) start: usize,
}

pub(crate) trait Element {
    const NUM_ELEMENTS: usize;
    const NUM_VERTICES: usize;
    const POINTS_PER_VERTEX: usize;
    const ELEMENTS: &'static [u32];
    const VERTEX_ATTRIBUTES: &'static [VertexAttrs];

    fn write_vertex_data(&self, buffer: &mut Vec<f32>);
}

pub(crate) struct ElemArray<E: Element> {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    capacity: usize,
    vertex_buffer: Vec<f32>,
    phantom: PhantomData<E>,
}

impl<E: Element> ElemArray<E> {
    pub(crate) fn new(capacity: usize) -> ElemArray<E> {
        let (mut vao, mut vbo, mut ebo) = (0, 0, 0);
        let vbo_size = capacity * E::NUM_VERTICES * E::POINTS_PER_VERTEX;
        let ebo_size = capacity * E::NUM_ELEMENTS;
        let mut element_buffer = Vec::with_capacity(ebo_size);
        for i in 0..capacity {
            for elem in E::ELEMENTS {
                element_buffer.push(elem + (i * E::NUM_VERTICES) as u32);
            }
        }
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vbo_size * std::mem::size_of::<f32>()) as isize,
                std::ptr::null(),
                gl::STREAM_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ebo_size * std::mem::size_of::<u32>()) as isize,
                element_buffer.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            for (i, attrib) in E::VERTEX_ATTRIBUTES.iter().enumerate() {
                gl::VertexAttribPointer(
                    i as u32,
                    attrib.size,
                    gl::FLOAT,
                    gl::FALSE,
                    (attrib.stride * std::mem::size_of::<f32>()) as i32,
                    (attrib.start * std::mem::size_of::<f32>()) as *const _,
                )
            }
        }
        let mut ret = ElemArray {
            vao,
            vbo,
            ebo,
            capacity,
            vertex_buffer: Vec::new(),
            phantom: PhantomData,
        };
        ret.unbind();
        ret
    }

    pub(crate) fn push(&mut self, element: E) {
        element.write_vertex_data(&mut self.vertex_buffer);
    }

    pub(crate) fn flush(&mut self, _shader: &ActiveShaderProgram) {
        let vbo_size = self.capacity * E::NUM_VERTICES * E::POINTS_PER_VERTEX;
        let vertex_buffer_len = self.vertex_buffer.len();
        self.bind();
        for range in util::range::split(0..vertex_buffer_len, vbo_size) {
            let num_elems =
                (range.len() * E::NUM_ELEMENTS) / (E::NUM_VERTICES * E::POINTS_PER_VERTEX);
            unsafe {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (range.len() * std::mem::size_of::<f32>()) as isize,
                    self.vertex_buffer[range.start..].as_ptr() as *const _,
                );
                gl::DrawElements(
                    gl::TRIANGLES,
                    num_elems as i32,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                )
            }
        }
        self.unbind();
        self.vertex_buffer.clear();
    }

    fn bind(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        }
    }

    fn unbind(&mut self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}

impl<E: Element> Drop for ElemArray<E> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.ebo);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::slice;

    use euclid::{point2, size2, Rect, UnknownUnit};
    use gl::types::*;
    use util::hash::{FnvHashMap, FnvHashSet};

    use super::*;
    use crate::gl::shader::ShaderProgram;
    use crate::gl::tests::DummyGlLoader;

    thread_local! {
        static NEXT_OBJECT: RefCell<GLuint> = RefCell::new(1);
        static CUR_VAO: RefCell<GLuint> = RefCell::new(0);
        static CUR_VBO: RefCell<GLuint> = RefCell::new(0);
        static CUR_EBO: RefCell<GLuint> = RefCell::new(0);
        static ALLOCED_OBJECTS: RefCell<FnvHashSet<GLuint>> = RefCell::default();
        static VERTEX_BUFFERS: RefCell<FnvHashMap<GLuint, Vec<f32>>> = RefCell::default();
        static ELEMENT_BUFFERS: RefCell<FnvHashMap<GLuint, Vec<u32>>> = RefCell::default();
        static FLUSHED: RefCell<Vec<Vec<f32>>> = RefCell::default();
    }

    fn reset_state() {
        NEXT_OBJECT.with(|next_object| *next_object.borrow_mut() = 1);
        CUR_VAO.with(|vao| *vao.borrow_mut() = 0);
        CUR_VBO.with(|vbo| *vbo.borrow_mut() = 0);
        CUR_EBO.with(|ebo| *ebo.borrow_mut() = 0);
        ALLOCED_OBJECTS.with(|alloced| alloced.borrow_mut().clear());
        VERTEX_BUFFERS.with(|buffers| buffers.borrow_mut().clear());
        ELEMENT_BUFFERS.with(|buffers| buffers.borrow_mut().clear());
        FLUSHED.with(|flushed| flushed.borrow_mut().clear());
    }

    fn alloc_object() -> GLuint {
        NEXT_OBJECT.with(|next_object| {
            let mut next_object = next_object.borrow_mut();
            let ret = *next_object;
            ALLOCED_OBJECTS.with(|alloced| {
                alloced.borrow_mut().insert(*next_object);
            });
            *next_object = *next_object + 1;
            ret
        })
    }

    fn is_alloced(obj: u32) -> bool {
        ALLOCED_OBJECTS.with(|alloced| alloced.borrow().contains(&obj))
    }

    unsafe fn gen_buffer(n: i32, buffer: *mut GLuint) {
        assert!(n > 0 && !buffer.is_null());
        for i in 0..n {
            *buffer.offset(i as isize) = alloc_object();
        }
    }

    fn free_object(obj: u32) {
        ALLOCED_OBJECTS.with(|alloced| {
            let mut alloced = alloced.borrow_mut();
            assert!(alloced.remove(&obj));
        });
        VERTEX_BUFFERS.with(|buffers| {
            buffers.borrow_mut().remove(&obj);
        });
        ELEMENT_BUFFERS.with(|buffers| {
            buffers.borrow_mut().remove(&obj);
        });
    }

    unsafe fn delete_objects(n: i32, objects: *const GLuint) {
        assert!(n > 0 && !objects.is_null());
        for i in 0..n {
            free_object(*objects.offset(i as isize));
        }
    }

    unsafe fn bind_vertex_array(array: u32) {
        assert!(array == 0 || is_alloced(array));
        CUR_VAO.with(|vao| {
            *vao.borrow_mut() = array;
        });
    }

    unsafe fn bind_buffer(target: GLenum, buffer: GLuint) {
        assert!(buffer == 0 || is_alloced(buffer));
        match target {
            gl::ARRAY_BUFFER => {
                CUR_VBO.with(|vbo| {
                    *vbo.borrow_mut() = buffer;
                });
            }
            gl::ELEMENT_ARRAY_BUFFER => {
                CUR_EBO.with(|ebo| {
                    *ebo.borrow_mut() = buffer;
                });
            }
            _ => panic!("unexpected target: {}", target),
        }
    }

    unsafe fn buffer_data(target: GLenum, size: isize, data: *const c_void, _usage: GLenum) {
        match target {
            gl::ARRAY_BUFFER => {
                let size = size as usize / std::mem::size_of::<f32>();
                let vbo = CUR_VBO.with(|vbo| *vbo.borrow());
                assert_ne!(vbo, 0);
                assert!(is_alloced(vbo));
                VERTEX_BUFFERS.with(|buffers| {
                    let mut new_buf = vec![0.0; size];
                    if !data.is_null() {
                        new_buf.copy_from_slice(slice::from_raw_parts(data as *const f32, size));
                    }
                    let mut buffers = buffers.borrow_mut();
                    buffers.insert(vbo, new_buf);
                });
            }
            gl::ELEMENT_ARRAY_BUFFER => {
                let size = size as usize / std::mem::size_of::<f32>();
                let ebo = CUR_EBO.with(|ebo| *ebo.borrow());
                assert_ne!(ebo, 0);
                assert!(is_alloced(ebo));
                ELEMENT_BUFFERS.with(|buffers| {
                    let mut new_buf = vec![0; size];
                    if !data.is_null() {
                        new_buf.copy_from_slice(slice::from_raw_parts(data as *const u32, size));
                    }
                    let mut buffers = buffers.borrow_mut();
                    buffers.insert(ebo, new_buf);
                });
            }
            _ => panic!("unexpected target: {}", target),
        }
    }

    unsafe fn buffer_sub_data(target: GLenum, offset: isize, size: isize, data: *const c_void) {
        assert!(!data.is_null());
        match target {
            gl::ARRAY_BUFFER => {
                let size = size as usize / std::mem::size_of::<f32>();
                let offset = offset as usize / std::mem::size_of::<f32>();
                let vbo = CUR_VBO.with(|vbo| *vbo.borrow());
                assert_ne!(vbo, 0);
                assert!(is_alloced(vbo));
                VERTEX_BUFFERS.with(|buffers| {
                    let mut buffers = buffers.borrow_mut();
                    buffers.get_mut(&vbo).unwrap()[offset..offset + size]
                        .copy_from_slice(slice::from_raw_parts(data as *const f32, size));
                });
            }
            gl::ELEMENT_ARRAY_BUFFER => {
                let size = size as usize / std::mem::size_of::<f32>();
                let offset = offset as usize / std::mem::size_of::<u32>();
                let ebo = CUR_EBO.with(|ebo| *ebo.borrow());
                assert_ne!(ebo, 0);
                assert!(is_alloced(ebo));
                ELEMENT_BUFFERS.with(|buffers| {
                    let mut buffers = buffers.borrow_mut();
                    buffers.get_mut(&ebo).unwrap()[offset..offset + size]
                        .copy_from_slice(slice::from_raw_parts(data as *const u32, size));
                });
            }
            _ => panic!("unexpected target: {}", target),
        }
    }

    unsafe fn draw_elements(mode: GLenum, count: i32, typ: GLenum, _indices: *const c_void) {
        assert_eq!(mode, gl::TRIANGLES);
        assert_eq!(typ, gl::UNSIGNED_INT);
        assert!(count > 0);
        let count = count as usize;
        let mut dest = Vec::new();
        let ebo = CUR_EBO.with(|ebo| *ebo.borrow());
        assert!(ebo != 0 && is_alloced(ebo));
        let vbo = CUR_VBO.with(|vbo| *vbo.borrow());
        assert!(vbo != 0 && is_alloced(vbo));
        ELEMENT_BUFFERS.with(|elem_bufs| {
            let elem_bufs = elem_bufs.borrow();
            let ebuf = elem_bufs.get(&ebo).unwrap();
            VERTEX_BUFFERS.with(|vertex_bufs| {
                let vertex_bufs = vertex_bufs.borrow();
                let vbuf = vertex_bufs.get(&vbo).unwrap();
                for i in 0..count {
                    // NOTE: stride here is assumed to be 2, because we're using Square with 2
                    //       f32s per vertex
                    assert!(ebuf[i] as usize * 2 + 1 < vbuf.len());
                    dest.push(vbuf[ebuf[i] as usize * 2]);
                    dest.push(vbuf[ebuf[i] as usize * 2 + 1]);
                }
            });
        });
        FLUSHED.with(|flushed| {
            flushed.borrow_mut().push(dest);
        });
    }

    fn test_prologue() {
        reset_state();
        let mut loader = DummyGlLoader::default();
        loader.BindVertexArray = bind_vertex_array;
        loader.BindBuffer = bind_buffer;
        loader.BufferData = buffer_data;
        loader.BufferSubData = buffer_sub_data;
        loader.DeleteBuffers = delete_objects;
        loader.DeleteVertexArrays = delete_objects;
        loader.DrawElements = draw_elements;
        loader.GenBuffers = gen_buffer;
        loader.GenVertexArrays = gen_buffer;
        loader.load_all();
    }

    struct Square(Rect<f32, UnknownUnit>);

    impl Element for Square {
        const NUM_ELEMENTS: usize = 6;
        const NUM_VERTICES: usize = 4;
        const POINTS_PER_VERTEX: usize = 2;
        const ELEMENTS: &'static [u32] = &[0, 2, 1, 1, 2, 3];
        const VERTEX_ATTRIBUTES: &'static [VertexAttrs] = &[VertexAttrs {
            size: 2,
            stride: 2,
            start: 0,
        }];

        fn write_vertex_data(&self, buffer: &mut Vec<f32>) {
            buffer.extend_from_slice(&[
                self.0.min_x(),
                self.0.min_y(),
                self.0.min_x(),
                self.0.max_y(),
                self.0.max_x(),
                self.0.min_y(),
                self.0.max_x(),
                self.0.max_y(),
            ]);
        }
    }

    #[test]
    fn element_buffer() {
        test_prologue();
        let arr = ElemArray::<Square>::new(3);
        assert!(ELEMENT_BUFFERS.with(|ebuf| {
            ebuf.borrow().get(&arr.ebo).unwrap()
                == &[0, 2, 1, 1, 2, 3, 4, 6, 5, 5, 6, 7, 8, 10, 9, 9, 10, 11]
        }));
    }

    #[test]
    fn bind_unbind_drop() {
        test_prologue();
        {
            let mut arr = ElemArray::<Square>::new(2);
            ALLOCED_OBJECTS.with(|alloced| {
                let alloced = alloced.borrow();
                assert!(alloced.contains(&1));
                assert!(alloced.contains(&2));
                assert!(alloced.contains(&3));
            });
            assert_eq!(CUR_VAO.with(|vao| *vao.borrow()), 0);
            assert_eq!(CUR_VBO.with(|vbo| *vbo.borrow()), 0);
            assert_eq!(CUR_EBO.with(|ebo| *ebo.borrow()), 0);
            arr.bind();
            assert_eq!(CUR_VAO.with(|vao| *vao.borrow()), 1);
            assert_eq!(CUR_VBO.with(|vbo| *vbo.borrow()), 2);
            assert_eq!(CUR_EBO.with(|ebo| *ebo.borrow()), 3);
            arr.unbind();
            assert_eq!(CUR_VAO.with(|vao| *vao.borrow()), 0);
            assert_eq!(CUR_VBO.with(|vbo| *vbo.borrow()), 0);
            assert_eq!(CUR_EBO.with(|ebo| *ebo.borrow()), 0);
        }
        ALLOCED_OBJECTS.with(|alloced| {
            let alloced = alloced.borrow();
            assert!(alloced.is_empty())
        });
    }

    #[test]
    fn push_flush() {
        test_prologue();
        let squares = [
            Square(Rect::new(point2(0.0, 1.0), size2(2.0, 3.0))),
            Square(Rect::new(point2(4.0, 5.0), size2(6.0, 7.0))),
            Square(Rect::new(point2(8.0, 9.0), size2(10.0, 11.0))),
        ];
        let mut arr = ElemArray::<Square>::new(2);
        for square in squares {
            arr.push(square);
        }
        assert_eq!(
            arr.vertex_buffer,
            &[
                0.0, 1.0, 0.0, 4.0, 2.0, 1.0, 2.0, 4.0, 4.0, 5.0, 4.0, 12.0, 10.0, 5.0, 10.0, 12.0,
                8.0, 9.0, 8.0, 20.0, 18.0, 9.0, 18.0, 20.0,
            ]
        );
        let mut shader = ShaderProgram::new("abc", "def").unwrap();
        arr.flush(&shader.use_program());
        FLUSHED.with(|flushed| {
            assert_eq!(
                *flushed.borrow(),
                vec![
                    vec![
                        0.0, 1.0, 2.0, 1.0, 0.0, 4.0, 0.0, 4.0, 2.0, 1.0, 2.0, 4.0, 4.0, 5.0, 10.0,
                        5.0, 4.0, 12.0, 4.0, 12.0, 10.0, 5.0, 10.0, 12.0,
                    ],
                    vec![8.0, 9.0, 18.0, 9.0, 8.0, 20.0, 8.0, 20.0, 18.0, 9.0, 18.0, 20.0,]
                ]
            );
        });
    }
}
