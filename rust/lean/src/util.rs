use std::marker::PhantomData;

// Reference:
// <https://stackoverflow.com/questions/62713667/how-to-implement-send-or-sync-for-a-type>
// This is a workaround until [negative
// impls](https://github.com/rust-lang/rust/issues/68318) are stable.
pub type NonSendNonSync = PhantomData<*const ()>;
