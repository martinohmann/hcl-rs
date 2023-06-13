macro_rules! forward_decorate_impl {
    ($($ty:ident => { $($variant:ident),+ }),+ $(,)?) => {
        $(
            impl $crate::Decorate for $ty {
                fn decor(&self) -> &$crate::Decor {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::Decorate::decor(v),
                        )*
                    }
                }

                fn decor_mut(&mut self) -> &mut $crate::Decor {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::Decorate::decor_mut(v),
                        )*
                    }
                }
            }
        )+
    };
}

macro_rules! forward_span_impl {
    ($($ty:ident => { $($variant:ident),+ }),+ $(,)?) => {
        $(
            impl $crate::Span for $ty {
                fn span(&self) -> Option<std::ops::Range<usize>> {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::Span::span(v),
                        )*
                    }
                }
            }

            impl $crate::SetSpan for $ty {
                fn set_span(&mut self, span: std::ops::Range<usize>) {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::SetSpan::set_span(v, span),
                        )*
                    }
                }
            }
        )+
    };
}

macro_rules! decorate_impl {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl $crate::Decorate for $ty {
                fn decor(&self) -> &$crate::Decor {
                    &self.decor
                }

                fn decor_mut(&mut self) -> &mut $crate::Decor {
                    &mut self.decor
                }
            }
        )+
    };
}

macro_rules! span_impl {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl $crate::Span for $ty {
                fn span(&self) -> Option<std::ops::Range<usize>> {
                    self.span.clone()
                }
            }

            impl $crate::SetSpan for $ty {
                fn set_span(&mut self, span: std::ops::Range<usize>) {
                    self.span = Some(span);
                }
            }
        )+
    };
}
