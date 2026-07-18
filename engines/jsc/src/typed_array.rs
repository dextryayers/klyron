pub enum TypedArrayKind {
    Int8,
    Int16,
    Int32,
    Uint8,
    Uint8Clamped,
    Uint16,
    Uint32,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

pub struct JSCTypedArray<T> {
    data: Vec<T>,
    kind: TypedArrayKind,
}

impl<T: Clone + Default> JSCTypedArray<T> {
    pub fn new(kind: TypedArrayKind, length: usize) -> Self {
        Self {
            data: vec![T::default(); length],
            kind,
        }
    }

    pub fn from_vec(kind: TypedArrayKind, data: Vec<T>) -> Self {
        Self { data, kind }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn set(&mut self, index: usize, value: T) -> Option<()> {
        if index < self.data.len() {
            self.data[index] = value;
            Some(())
        } else {
            None
        }
    }

    pub fn kind(&self) -> &TypedArrayKind {
        &self.kind
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.data.clone()
    }
}

pub type JSCInt8Array = JSCTypedArray<i8>;
pub type JSCInt16Array = JSCTypedArray<i16>;
pub type JSCInt32Array = JSCTypedArray<i32>;
pub type JSCUint8Array = JSCTypedArray<u8>;
pub type JSCUint8ClampedArray = JSCTypedArray<u8>;
pub type JSCUint16Array = JSCTypedArray<u16>;
pub type JSCUint32Array = JSCTypedArray<u32>;
pub type JSCFloat32Array = JSCTypedArray<f32>;
pub type JSCFloat64Array = JSCTypedArray<f64>;
