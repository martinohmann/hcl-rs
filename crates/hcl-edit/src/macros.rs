macro_rules! forward_decorate_impl {
    ($($ty:ident => { $($variant:ident),+ }),+ $(,)?) => {
        $(
            impl $crate::repr::Decorate for $ty {
                fn decor(&self) -> &$crate::repr::Decor {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::repr::Decorate::decor(v),
                        )*
                    }
                }

                fn decor_mut(&mut self) -> &mut $crate::repr::Decor {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::repr::Decorate::decor_mut(v),
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
            impl $crate::repr::Span for $ty {
                fn span(&self) -> Option<std::ops::Range<usize>> {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::repr::Span::span(v),
                        )*
                    }
                }
            }

            impl $crate::repr::SetSpan for $ty {
                fn set_span(&mut self, span: std::ops::Range<usize>) {
                    match self {
                        $(
                            $ty::$variant(v) => $crate::repr::SetSpan::set_span(v, span),
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
            impl $crate::repr::Decorate for $ty {
                fn decor(&self) -> &$crate::repr::Decor {
                    &self.decor
                }

                fn decor_mut(&mut self) -> &mut $crate::repr::Decor {
                    &mut self.decor
                }
            }
        )+
    };
}

macro_rules! span_impl {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl $crate::repr::Span for $ty {
                fn span(&self) -> Option<std::ops::Range<usize>> {
                    self.span.clone()
                }
            }

            impl $crate::repr::SetSpan for $ty {
                fn set_span(&mut self, span: std::ops::Range<usize>) {
                    self.span = Some(span);
                }
            }
        )+
    };
}
