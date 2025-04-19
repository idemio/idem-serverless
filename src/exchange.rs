use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use std::pin::Pin;
use std::sync::Arc;

pub trait Handler<I, O, M>: Send
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut Exchange<I, O, M>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}

pub struct Exchange<I, O, M>
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{
    metadata: Option<M>,
    input: I,
    output: O,
    input_listeners: Vec<Callback<Self>>,
    output_listeners: Vec<Callback<Self>>,
    attachments: HashMap<(AttachmentKey, TypeId), Box<dyn Any + Send>>,
}

impl<I, O, M> Exchange<I, O, M>
where
    I: Default + Send,
    O: Default + Send,
    M: Send,
{
    pub fn new() -> Self {
        Self {
            metadata: None,
            input: I::default(),
            output: O::default(),
            input_listeners: vec![],
            output_listeners: vec![],
            attachments: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, metadata: M) {
        self.metadata = Some(metadata);
    }

    pub fn add_attachment<K>(&mut self, key: AttachmentKey, value: Box<dyn Any + Send>)
    where
        K: Send + 'static
    {
        let type_id = TypeId::of::<K>();
        self.attachments.insert((key, type_id), value);
    }

    pub fn attachment<K>(&self, key: AttachmentKey) -> Option<&K>
    where
        K: Send + 'static,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get(&(key, type_id)) {
            option_any.downcast_ref::<K>()
        } else {
            None
        }
    }

    pub fn attachment_mut<K>(&mut self, key: AttachmentKey) -> Option<&mut K>
    where
        K: Send + 'static,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get_mut(&(key, type_id)) {
            option_any.downcast_mut::<K>()
        } else {
            None
        }
    }

    pub fn add_input_listener(&mut self, callback: impl Fn(Box<&Self>) + Send + 'static)
    where
        Self: Send,
        Self: Sized,
    {
        self.input_listeners.push(Callback::new(callback))
    }

    pub fn add_output_listener(&mut self, callback: impl Fn(Box<&Self>) + Send + 'static)
    where
        Self: Send,
        Self: Sized,
    {
        self.output_listeners.push(Callback::new(callback))
    }

    fn execute_input_listeners(&mut self) -> Result<(), ()> {
        self.execute_callbacks(&self.input_listeners)
    }

    fn execute_output_listeners(&mut self) -> Result<(), ()> {
        self.execute_callbacks(&self.output_listeners)
    }

    fn execute_callbacks(&self, callbacks: &Vec<Callback<Self>>) -> Result<(), ()>
    where
        Self: Send,
    {
        let mut pos = 0usize;
        while !callbacks.is_empty() && pos < callbacks.len() {
            log::trace!("Executing callback {}", pos);
            match callbacks.get(pos) {
                Some(callback) => callback.invoke(Box::new(self)),
                None => return Err(()),
            }
            pos += 1;
        }
        Ok(())
    }

    pub fn save_input(&mut self, request: I) {
        self.input = request;
    }

    pub fn input(&self) -> Result<&I, ()> {
        Ok(&self.input)
    }

    pub fn consume_request(&mut self) -> Result<I, ()> {
        match self.execute_input_listeners() {
            Ok(_) => {
                log::debug!("Successfully executed request listeners.");
                let consumed = std::mem::take(&mut self.input);
                Ok(consumed)
            }
            Err(_) => Err(()),
        }
    }

    pub fn save_output(&mut self, response: O) {
        self.output = response;
    }

    pub fn consume_output(&mut self) -> Result<O, ()> {
        match self.execute_output_listeners() {
            Ok(_) => {
                log::debug!("Successfully executed response listeners.");
                let consumed = std::mem::take(&mut self.output);
                Ok(consumed)
            }
            Err(_) => Err(()),
        }
    }
}

/* I wanted to make this struct use TypeId::of::<>() but it's not stable. */
#[derive(PartialOrd, PartialEq, Hash, Eq)]
pub struct AttachmentKey(pub u32);

impl AttachmentKey {
    /* common attachment keys */
    pub const APP_CONTEXT: AttachmentKey = AttachmentKey(1);
    pub const CLIENT_SRC: AttachmentKey = AttachmentKey(2);
    pub const CACHED_BODY: AttachmentKey = AttachmentKey(3);
}

pub struct Callback<T: Send + ?Sized> {
    callback: Box<dyn Fn(Box<&T>) + Send>,
}
impl<T: Send + ?Sized> Callback<T> {
    pub fn new(callback: impl Fn(Box<&T>) + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }

    pub fn invoke(&self, context: Box<&T>) {
        (self.callback)(context);
    }
}
