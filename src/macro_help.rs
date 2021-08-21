#[macro_export]
macro_rules! count_idents {
    ($($idents:ident),* $(,)*) => {
        {
            #[derive(Copy, Clone)]
            #[allow(dead_code, non_camel_case_types)]
            
            enum IdentsCounter { $($idents,)* __CountIdentsLast }
            IdentsCounter::__CountIdentsLast as usize
        }
    };
}