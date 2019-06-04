
pub struct FieldMeta {
    offset: usize
}

pub unsafe trait Field {
    type Parent: ?Sized;
    type Type: ?Sized;
    const META: FieldMeta;
}

pub unsafe trait PinSafeField: Field {}

impl FieldMeta {
    pub const unsafe fn new_unchecked(offset: usize) -> Self {
        Self { offset }
    }
}

pub trait Project<F: Field> {
    type Projection;

    fn project(self, field: F) -> Self::Projection;
}

pub trait RawPtrExt<F: Field<Parent = Self>> {
    unsafe fn project_inbounds(ptr: *const Self, field: F) -> *const F::Type;

    unsafe fn project_inbounds_mut(ptr: *mut Self, field: F) -> *mut F::Type;
}

impl<F: Field> RawPtrExt<F> for F::Parent where F::Type: Sized {
    unsafe fn project_inbounds(ptr: *const Self, _: F) -> *const F::Type {
        (ptr as *const u8)
            .add(F::META.offset)
            as *const F::Type
    }

    unsafe fn project_inbounds_mut(ptr: *mut Self, _: F) -> *mut F::Type {
        (ptr as *mut u8)
            .add(F::META.offset)
            as *mut F::Type
    }
}

impl<F: Field> Project<F> for *const F::Parent
where F::Type: Sized {
    type Projection = *const F::Type;

    fn project(self, _: F) -> Self::Projection {
        (self as *const u8)
            .wrapping_add(F::META.offset)
            as *const F::Type
    }
}

impl<F: Field> Project<F> for *mut F::Parent
where F::Type: Sized {
    type Projection = *mut F::Type;

    fn project(self, _: F) -> Self::Projection {
        (self as *mut u8)
            .wrapping_add(F::META.offset)
            as *mut F::Type
    }
}

impl<'a, F: Field> Project<F> for &'a F::Parent
where F::Type: 'a + Sized {
    type Projection = &'a F::Type;

    fn project(self, field: F) -> Self::Projection {
        unsafe {
            let ptr: *const F::Parent = self;

            let ptr = RawPtrExt::project_inbounds(ptr, field);

            &*ptr
        }
    }
}

impl<'a, F: Field> Project<F> for &'a mut F::Parent
where F::Type: 'a + Sized {
    type Projection = &'a mut F::Type;

    fn project(self, field: F) -> Self::Projection {
        unsafe {
            let ptr: *mut F::Parent = self;

            let ptr = RawPtrExt::project_inbounds_mut(ptr, field);

            &mut *ptr
        }
    }
}

use std::cell::{Ref, RefMut};

impl<'a, F: Field> Project<F> for Ref<'a, F::Parent>
where F::Type: 'a + Sized {
    type Projection = Ref<'a, F::Type>;
    
    fn project(self, field: F) -> Self::Projection {
        Ref::map(self, |x| x.project(field))
    }
}

impl<'a, F: Field> Project<F> for RefMut<'a, F::Parent>
where F::Type: 'a + Sized {
    type Projection = RefMut<'a, F::Type>;
    
    fn project(self, field: F) -> Self::Projection {
        RefMut::map(self, |x| x.project(field))
    }
}

use std::pin::Pin;

impl<'a, F: PinSafeField> Project<F> for Pin<&'a F::Parent>
where F::Type: 'a + Sized, {
    type Projection = Pin<&'a F::Type>;
    
    fn project(self, field: F) -> Self::Projection {
        unsafe {
            Pin::map_unchecked(
                self,
                |x| x.project(field)
            )
        }
    }
}

impl<'a, F: PinSafeField> Project<F> for Pin<&'a mut F::Parent>
where F::Type: 'a + Sized {
    type Projection = Pin<&'a mut F::Type>;
    
    fn project(self, field: F) -> Self::Projection {
        unsafe {
            Pin::map_unchecked_mut(
                self,
                |x| x.project(field)
            )
        }
    }
}
