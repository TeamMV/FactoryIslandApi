use abi_stable::std_types::RVec;
use crate::mods::modsdk::MOpt;

#[macro_export]
macro_rules! rvec_save {
    ($this: ident => $($field:ident,)*) => {
        {
            use mvutils::bytebuffer::ByteBufferExtras;
            use mvutils::save::Savable;
            use abi_stable::traits::IntoReprC;
            let mut buffer = bytebuffer::ByteBuffer::new_le();
            $(
                $this.$field.save(&mut buffer);
            )*
            let v = buffer.into_vec().into_c();
            v
        }
    };
}

#[macro_export]
macro_rules! rvec_load {
    (
        $vec:ident for $this:ident => {
            $($load_field:ident : $t:ty ,)*
        }
    ) => {{
        use mvutils::bytebuffer::ByteBufferExtras;
        use mvutils::save::Savable;
        use crate::mods::modsdk::ToMOpt;
        let vec = RVec::to_vec(&$vec);
        let mut buffer = bytebuffer::ByteBuffer::from_vec_le(vec);

        $(
            $this.$load_field =
                <$t>::load(&mut buffer)
                    .ok()
                    .to_m()?;
        )*
    }};
}

#[macro_export]
macro_rules! leak {
    ($e:expr) => {
        Box::into_raw(Box::new($e))
    };
}

#[macro_export]
macro_rules! p {
    ($r:ident) => {
        unsafe { $r as *mut _ as *mut () }
    };
    ($ptr:ident as $to:ty) => {
        unsafe { ($ptr as *mut $to).as_mut().expect("Bro this is illegal check ur cast") }
    };
}

#[macro_export]
macro_rules! ptr_invoke_clone {
    ($p:expr) => {
        {
            let cloned = unsafe { (*$p).clone() };
            leak!(cloned)
        }
    };
}

#[macro_export]
macro_rules! this {
    ($p:expr) => {
        unsafe { $p as *mut () }
    };
}