use winnow::stream::Located;

pub type Input<'a> = Located<&'a [u8]>;
