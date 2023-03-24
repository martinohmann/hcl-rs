macro_rules! forward_decorate_impl {
    ($($ty:ident => { $($variant:ident),+ }),+ $(,)?) => {
        $(
            impl Decorate for $ty {
                fn decor(&self) -> &Decor {
                    match self {
                        $(
                            $ty::$variant(v) => v.decor(),
                        )*
                    }
                }

                fn decor_mut(&mut self) -> &mut Decor {
                    match self {
                        $(
                            $ty::$variant(v) => v.decor_mut(),
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
            impl Span for $ty {
                fn span(&self) -> Option<Range<usize>> {
                    match self {
                        $(
                            $ty::$variant(v) => v.span(),
                        )*
                    }
                }
            }

            impl SetSpan for $ty {
                fn set_span(&mut self, span: Range<usize>) {
                    match self {
                        $(
                            $ty::$variant(v) => v.set_span(span),
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
            impl Decorate for $ty {
                fn decor(&self) -> &Decor {
                    &self.decor
                }

                fn decor_mut(&mut self) -> &mut Decor {
                    &mut self.decor
                }
            }
        )+
    };
}

macro_rules! span_impl {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl Span for $ty {
                fn span(&self) -> Option<Range<usize>> {
                    self.span.clone()
                }
            }

            impl SetSpan for $ty {
                fn set_span(&mut self, span: Range<usize>) {
                    self.span = Some(span);
                }
            }
        )+
    };
}
