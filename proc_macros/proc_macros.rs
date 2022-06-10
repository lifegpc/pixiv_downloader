extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

#[proc_macro]
pub fn define_struct_reader_fn(item: TokenStream) -> TokenStream {
    let i = parse_macro_input!(item as Ident);
    let lefname = format!("read_le_{}", i);
    let lefname = Ident::new(&lefname, i.span());
    let befname = format!("read_be_{}", i);
    let befname = Ident::new(&befname, i.span());
    let stream = quote! {
        #[doc = concat!("Read [", stringify!(#i), "] in little endian.")]
        fn #lefname(&mut self) -> Result<#i, Self::Error>;
        #[doc = concat!("Read [", stringify!(#i), "] in big endian.")]
        fn #befname(&mut self) -> Result<#i, Self::Error>;
    };
    stream.into()
}

#[proc_macro]
pub fn impl_struct_reader_read(item: TokenStream) -> TokenStream {
    let i = parse_macro_input!(item as Ident);
    let lefname = format!("read_le_{}", i);
    let lefname = Ident::new(&lefname, i.span());
    let befname = format!("read_be_{}", i);
    let befname = Ident::new(&befname, i.span());
    let stream = quote! {
        fn #lefname(&mut self) -> Result<#i, Self::Error> {
            let mut buf = [0u8; std::mem::size_of::<#i>()];
            self.read_exact(&mut buf)?;
            Ok(<#i>::from_le_bytes(buf))
        }
        fn #befname(&mut self) -> Result<#i, Self::Error> {
            let mut buf = [0u8; std::mem::size_of::<#i>()];
            self.read_exact(&mut buf)?;
            Ok(<#i>::from_be_bytes(buf))
        }
    };
    stream.into()
}

#[proc_macro]
pub fn define_struct_writer_fn(item: TokenStream) -> TokenStream {
    let i = parse_macro_input!(item as Ident);
    let lefname = format!("write_le_{}", i);
    let lefname = Ident::new(&lefname, i.span());
    let befname = format!("write_be_{}", i);
    let befname = Ident::new(&befname, i.span());
    let stream = quote! {
        #[doc = concat!("Write [", stringify!(#i), "] in little endian.")]
        fn #lefname(&mut self, data: #i) -> Result<(), Self::Error>;
        #[doc = concat!("Write [", stringify!(#i), "] in big endian.")]
        fn #befname(&mut self, data: #i) -> Result<(), Self::Error>;
    };
    stream.into()
}

#[proc_macro]
pub fn impl_struct_writer_write(item: TokenStream) -> TokenStream {
    let i = parse_macro_input!(item as Ident);
    let lefname = format!("write_le_{}", i);
    let lefname = Ident::new(&lefname, i.span());
    let befname = format!("write_be_{}", i);
    let befname = Ident::new(&befname, i.span());
    let stream = quote! {
        fn #lefname(&mut self, data: #i) -> Result<(), Self::Error> {
            self.write_all(&data.to_le_bytes())
        }
        fn #befname(&mut self, data: #i) -> Result<(), Self::Error> {
            self.write_all(&data.to_be_bytes())
        }
    };
    stream.into()
}
