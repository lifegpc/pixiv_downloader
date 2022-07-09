use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::token;
use syn::Block;
use syn::Expr;
use syn::Ident;
use syn::ItemFn;
use syn::Lit;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;

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
        #[inline]
        fn #lefname(&mut self, data: #i) -> Result<(), Self::Error> {
            self.write_all(&data.to_le_bytes())
        }
        #[inline]
        fn #befname(&mut self, data: #i) -> Result<(), Self::Error> {
            self.write_all(&data.to_be_bytes())
        }
    };
    stream.into()
}

#[proc_macro_attribute]
pub fn async_timeout_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let dura = parse_duration::parse(attr.to_string().as_str()).unwrap();
    let secs = LitInt::new(format!("{}", dura.as_secs()).as_str(), sig.ident.span());
    let nanos = LitInt::new(
        format!("{}", dura.subsec_nanos()).as_str(),
        sig.ident.span(),
    );
    let stmts = &block.stmts;
    let stream = quote! {
        #(#attrs)* #vis #sig {
            let dura = std::time::Duration::new(#secs, #nanos);
            let f = async {
                #(#stmts)*
            };
            tokio::time::timeout(dura, f).await.unwrap();
        }
    };
    stream.into()
}

struct FanboxApiTest {
    pub name: Ident,
    pub block: Block,
}

impl Parse for FanboxApiTest {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = Ident::parse(input)?;
        token::Comma::parse(input)?;
        let block = syn::Block::parse(input)?;
        Ok(Self { name, block })
    }
}

#[proc_macro]
pub fn fanbox_api_test(item: TokenStream) -> TokenStream {
    let FanboxApiTest { name, block } = parse_macro_input!(item as FanboxApiTest);
    let stmts = block.stmts;
    let stream = quote! {
        #[proc_macros::async_timeout_test(120s)]
        #[tokio::test(flavor = "multi_thread")]
        async fn #name() {
            match std::env::var("FANBOX_COOKIES_FILE") {
                Ok(path) => {
                    let client = crate::fanbox_api::FanboxClient::new();
                    if !client.init(Some(path)) {
                        panic!("Failed to initiailze the client.");
                    }
                    if !client.check_login().await {
                        println!("The client is not logined. Skip test.");
                        return;
                    }
                    #(#stmts)*
                }
                Err(_) => {
                    println!("No cookies files specified, skip test.")
                }
            }
        }
    };
    stream.into()
}

struct FanboxApiQuickTest {
    pub name: Ident,
    pub expr: Expr,
    pub errmsg: LitStr,
}

impl Parse for FanboxApiQuickTest {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = Ident::parse(input)?;
        token::Comma::parse(input)?;
        let expr = Expr::parse(input)?;
        token::Comma::parse(input)?;
        let errmsg = Lit::parse(input)?;
        match errmsg {
            Lit::Str(errmsg) => Ok(Self { name, expr, errmsg }),
            _ => Err(syn::Error::new(errmsg.span(), "Failed to parse string.")),
        }
    }
}

#[proc_macro]
pub fn fanbox_api_quick_test(item: TokenStream) -> TokenStream {
    let FanboxApiQuickTest { name, expr, errmsg } = parse_macro_input!(item as FanboxApiQuickTest);
    let stream = quote! {
        #[proc_macros::async_timeout_test(120s)]
        #[tokio::test(flavor = "multi_thread")]
        async fn #name() {
            match std::env::var("FANBOX_COOKIES_FILE") {
                Ok(path) => {
                    let client = crate::fanbox_api::FanboxClient::new();
                    if !client.init(Some(path)) {
                        panic!("Failed to initiailze the client.");
                    }
                    if !client.check_login().await {
                        println!("The client is not logined. Skip test.");
                        return;
                    }
                    match #expr.await {
                        Some(data) => {
                            println!("{:?}", data);
                        }
                        None => {
                            panic!("{}", #errmsg);
                        }
                    }
                }
                Err(_) => {
                    println!("No cookies files specified, skip test.")
                }
            }
        }
    };
    stream.into()
}

struct FilterHttpMethods {
    pub req: Ident,
    pub typ: Expr,
    pub handle_options: LitBool,
    pub ctx: Option<Expr>,
    pub methods: Vec<Ident>,
}

impl Parse for FilterHttpMethods {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let req = Ident::parse(input)?;
        token::Comma::parse(input)?;
        let typ = Expr::parse(input)?;
        token::Comma::parse(input)?;
        let mut methods = Vec::new();
        let handle_options = match Lit::parse(input)? {
            Lit::Bool(s) => s,
            _ => {
                return Err(syn::Error::new(input.span(), "Failed to parse boolean."));
            }
        };
        let ctx = if handle_options.value() {
            token::Comma::parse(input)?;
            Some(Expr::parse(input)?)
        } else {
            None
        };
        loop {
            if input.cursor().eof() {
                break;
            }
            token::Comma::parse(input)?;
            let method = Ident::parse(input)?;
            methods.push(method);
        }
        Ok(Self {
            req,
            typ,
            handle_options,
            ctx,
            methods,
        })
    }
}

/// Filter http methods.
///
/// `request, 405 body, handle_options, [method [, method [, method ...]]]`
#[proc_macro]
pub fn filter_http_methods(item: TokenStream) -> TokenStream {
    let FilterHttpMethods {
        req,
        typ,
        handle_options,
        ctx,
        methods,
    } = parse_macro_input!(item as FilterHttpMethods);
    let mut header_value = Vec::new();
    let mut streams = Vec::new();
    let mut enable_options = false;
    for method in methods {
        header_value.push(method.to_string());
        if method == "OPTIONS" && handle_options.value() {
            enable_options = true;
        } else {
            streams.push(quote!(&hyper::Method::#method => {}));
        }
    }
    let allow_header = header_value.join(", ");
    let allow_header = LitStr::new(allow_header.as_str(), req.span());
    if enable_options {
        streams.push(quote!(&hyper::Method::OPTIONS => {
            let builder = hyper::Response::builder();
            let headers = #req.headers();
            let origin = match headers.get(hyper::header::ORIGIN) {
                Some(origin) => match origin.to_str() {
                    Ok(origin) => Some(origin.to_owned()),
                    Err(_) => None,
                },
                None => None,
            };
            match origin {
                Some(origin) => {
                    match #ctx.cors.matches(origin.as_str()) {
                        crate::server::cors::CorsResult::Allowed => {
                            let builder = builder.header("Access-Control-Allow-Origin", origin.as_str());
                            return Ok(builder.status(200).header("Allow", #allow_header).body(#typ).unwrap());
                        }
                        crate::server::cors::CorsResult::AllowedAll => {
                            let builder = builder.header("Access-Control-Allow-Origin", "*");
                            return Ok(builder.status(200).header("Allow", #allow_header).body(#typ).unwrap());
                        }
                        _ => {
                            return Ok(builder.status(400).header("Allow", #allow_header).body(#typ).unwrap());
                        }
                    }
                }
                None => {
                    return Ok(builder.status(200).header("Allow", #allow_header).body(#typ).unwrap());
                }
            }
        }));
    }
    let post_stream = if enable_options {
        quote!(
            let mut builder = hyper::Response::builder();
            let headers = #req.headers();
            let origin = match headers.get(hyper::header::ORIGIN) {
                Some(origin) => match origin.to_str() {
                    Ok(origin) => Some(origin.to_owned()),
                    Err(_) => None,
                },
                None => None,
            };
            match origin {
                Some(origin) => {
                    match #ctx.cors.matches(origin.as_str()) {
                        crate::server::cors::CorsResult::Allowed => {
                            builder.headers_mut().unwrap().insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
                        }
                        crate::server::cors::CorsResult::AllowedAll => {
                            builder.headers_mut().unwrap().insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                        }
                        _ => {
                            return Ok(builder.status(403).body(#typ).unwrap());
                        }
                    }
                }
                None => {}
            }
        )
    } else {
        quote!()
    };
    let stream = quote! {
        match #req.method() {
            #(#streams)*
            _ => {
                return Ok(hyper::Response::builder().status(405).header("Allow", #allow_header).body(#typ).unwrap())
            }
        }
        #post_stream
    };
    stream.into()
}
