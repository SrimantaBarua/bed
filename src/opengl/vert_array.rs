// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr;

use gl::types::GLuint;

use crate::opengl::ActiveShaderProgram;

pub(crate) trait Element {
    // Number of vertices
    fn num_vertices() -> usize;

    // Number of elements
    fn num_elements() -> usize;

    // Elements for one entity at offset 0
    fn elements() -> &'static [u32];

    // Number of floats per vertex
    fn num_points_per_vertex() -> usize;

    // Vertex attributes (size, stride, start)
    fn vertex_attributes() -> &'static [(i32, usize, usize)];

    // Raw data
    fn data(&self) -> &[f32];
}

pub(crate) struct ElemArr<E>
where
    E: Element,
{
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    cap: usize,
    vbuf: Vec<f32>,
    phantom: PhantomData<E>,
}

impl<E> ElemArr<E>
where
    E: Element,
{
    pub(crate) fn new(cap: usize) -> ElemArr<E> {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;

        let num_vertices = E::num_vertices();
        let num_elements = E::num_elements();
        let points_per_vertex = E::num_points_per_vertex();
        let attribs = E::vertex_attributes();
        let elements = E::elements();

        let vbo_size = cap * num_vertices * points_per_vertex;
        let ebo_size = cap * num_elements;
        let mut ebuf = Vec::with_capacity(ebo_size);
        for i in 0..cap {
            for elem in elements {
                ebuf.push(elem + (i * num_vertices) as u32);
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
                (vbo_size * size_of::<f32>()) as isize,
                ptr::null(),
                gl::STREAM_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ebo_size * size_of::<u32>()) as isize,
                ebuf.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            for i in 0..attribs.len() {
                gl::VertexAttribPointer(
                    i as u32,
                    attribs[i].0,
                    gl::FLOAT,
                    gl::FALSE,
                    (attribs[i].1 * size_of::<f32>()) as i32,
                    (attribs[i].2 * size_of::<f32>()) as *const _,
                );
                gl::EnableVertexAttribArray(i as u32);
            }
        }
        ElemArr {
            vao,
            vbo,
            ebo,
            cap,
            vbuf: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub(crate) fn push(&mut self, elem: E) {
        self.vbuf.extend_from_slice(elem.data().as_ref());
    }

    pub(crate) fn flush(&mut self, _shader: &ActiveShaderProgram) {
        let mut vidx = 0;

        let num_vertices = E::num_vertices();
        let num_elements = E::num_elements();
        let points_per_vertex = E::num_points_per_vertex();

        let vbo_size = self.cap * num_vertices * points_per_vertex;
        let ebo_size = self.cap * num_elements;
        let vbuf_len = self.vbuf.len();
        self.bind();
        while vbuf_len > vidx + vbo_size {
            unsafe {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (vbo_size * size_of::<f32>()) as isize,
                    self.vbuf[vidx..].as_ptr() as *const _,
                );
                gl::DrawElements(
                    gl::TRIANGLES,
                    ebo_size as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
            vidx += vbo_size;
        }
        if vbuf_len > vidx {
            let num_elems = ((vbuf_len - vidx) / (num_vertices * points_per_vertex)) * num_elements;
            unsafe {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    ((vbuf_len - vidx) * size_of::<f32>()) as isize,
                    self.vbuf[vidx..].as_ptr() as *const _,
                );
                gl::DrawElements(
                    gl::TRIANGLES,
                    num_elems as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
        self.unbind();
        self.vbuf.clear();
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

impl<E> Drop for ElemArr<E>
where
    E: Element,
{
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.ebo);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
