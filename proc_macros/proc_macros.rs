use convert_case::Case;
use convert_case::Casing;
use proc_macro::TokenStream;
use quote::quote;
use syn::bracketed;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::token;
use syn::Block;
use syn::Expr;
use syn::Ident;
use syn::ItemEnum;
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

struct HTTPHeader {
    pub header: String,
}

impl Parse for HTTPHeader {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut header = String::new();
        let ident = Ident::parse(input)?;
        header += ident.to_string().replace("_", "-").as_str();
        loop {
            if input.cursor().eof() {
                break;
            }
            match token::Sub::parse(input) {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
            let ident = Ident::parse(input)?;
            header += "-";
            header += ident.to_string().replace("_", "-").as_str();
        }
        return Ok(Self { header });
    }
}

struct FilterHttpMethods {
    pub req: Ident,
    pub typ: Expr,
    pub handle_options: LitBool,
    pub ctx: Option<Expr>,
    pub methods: Vec<Ident>,
    pub cors_methods: Option<Vec<Ident>>,
    pub expose_headers: Option<Vec<String>>,
    pub cors_allow_headers: Option<Vec<String>>,
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
        let mut cors_methods = None;
        let mut expose_headers = None;
        let mut cors_allow_headers = None;
        loop {
            if input.cursor().eof() {
                break;
            }
            token::Comma::parse(input)?;
            let method = Ident::parse(input)?;
            if method.to_string() == "cors_methods" {
                cors_methods.replace(Vec::new());
                token::Eq::parse(input)?;
                let content;
                bracketed!(content in input);
                let first: Ident = content.parse()?;
                cors_methods.as_mut().unwrap().push(first);
                while !content.is_empty() {
                    let _: token::Comma = content.parse()?;
                    let m: Ident = content.parse()?;
                    cors_methods.as_mut().unwrap().push(m);
                }
            } else if method.to_string() == "expose_headers" {
                expose_headers.replace(Vec::new());
                token::Eq::parse(input)?;
                let content;
                bracketed!(content in input);
                let first: HTTPHeader = content.parse()?;
                expose_headers.as_mut().unwrap().push(first.header);
                while !content.is_empty() {
                    let _: token::Comma = content.parse()?;
                    let m: HTTPHeader = content.parse()?;
                    expose_headers.as_mut().unwrap().push(m.header);
                }
            } else if method.to_string() == "allow_headers" {
                cors_allow_headers.replace(Vec::new());
                token::Eq::parse(input)?;
                let content;
                bracketed!(content in input);
                let first: HTTPHeader = content.parse()?;
                cors_allow_headers.as_mut().unwrap().push(first.header);
                while !content.is_empty() {
                    let _: token::Comma = content.parse()?;
                    let m: HTTPHeader = content.parse()?;
                    cors_allow_headers.as_mut().unwrap().push(m.header);
                }
            } else {
                methods.push(method);
            }
        }
        Ok(Self {
            req,
            typ,
            handle_options,
            ctx,
            methods,
            cors_methods,
            expose_headers,
            cors_allow_headers,
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
        cors_methods,
        expose_headers,
        cors_allow_headers,
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
    let cors_methods_header = match cors_methods {
        Some(methods) => {
            let mut v = Vec::new();
            for method in methods {
                v.push(method.to_string());
            }
            v.join(", ")
        }
        None => allow_header.clone(),
    };
    let allow_header = LitStr::new(allow_header.as_str(), req.span());
    let cors_methods_header = LitStr::new(cors_methods_header.as_str(), req.span());
    if enable_options {
        let expose_headers = match &expose_headers {
            Some(h) => {
                let headers = h.join(", ");
                let headers = LitStr::new(headers.as_str(), req.span());
                quote!(
                    let builder = builder.header(hyper::header::ACCESS_CONTROL_EXPOSE_HEADERS, #headers);
                )
            }
            None => quote!(),
        };
        let cors_allow_headers = match &cors_allow_headers {
            Some(h) => {
                let headers = h.join(", ");
                let headers = LitStr::new(headers.as_str(), req.span());
                quote!(
                    let builder = builder.header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, #headers);
                )
            }
            None => quote!(),
        };
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
                            let builder = builder
                                .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.as_str())
                                .header(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, #cors_methods_header);
                                #expose_headers
                                #cors_allow_headers
                            return Ok(builder.status(200).header("Allow", #allow_header).body(#typ)?);
                        }
                        crate::server::cors::CorsResult::AllowedAll => {
                            let builder = builder
                                .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                .header(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, #cors_methods_header);
                                #expose_headers
                                #cors_allow_headers
                            return Ok(builder.status(200).header("Allow", #allow_header).body(#typ)?);
                        }
                        _ => {
                            return Ok(builder.status(400).header("Allow", #allow_header).body(#typ)?);
                        }
                    }
                }
                None => {
                    return Ok(builder.status(200).header("Allow", #allow_header).body(#typ)?);
                }
            }
        }));
    }
    let post_stream = if enable_options {
        let expose_headers = match expose_headers {
            Some(h) => {
                let headers = h.join(", ");
                let headers = LitStr::new(headers.as_str(), req.span());
                quote!(
                    builder.
                        headers_mut()
                        .try_err(gettext("Failed to build response."))?
                        .insert(hyper::header::ACCESS_CONTROL_EXPOSE_HEADERS, #headers.parse()?);
                )
            }
            None => quote!(),
        };
        let cors_allow_headers = match cors_allow_headers {
            Some(h) => {
                let headers = h.join(", ");
                let headers = LitStr::new(headers.as_str(), req.span());
                quote!(
                    builder.
                        headers_mut()
                        .try_err(gettext("Failed to build response."))?
                        .insert(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, #headers.parse()?);
                )
            }
            None => quote!(),
        };
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
                    use crate::ext::try_err::TryErr;
                    use crate::gettext;
                    match #ctx.cors.matches(origin.as_str()) {
                        crate::server::cors::CorsResult::Allowed => {
                            builder
                                .headers_mut()
                                .try_err(gettext("Failed to build response."))?
                                .insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse()?);
                            #expose_headers
                            #cors_allow_headers
                        }
                        crate::server::cors::CorsResult::AllowedAll => {
                            builder
                                .headers_mut()
                                .try_err(gettext("Failed to build response."))?
                                .insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse()?);
                            #expose_headers
                            #cors_allow_headers
                        }
                        _ => {
                            return Ok(builder.status(403).body(#typ)?);
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
                return Ok(hyper::Response::builder().status(405).header("Allow", #allow_header).body(#typ)?)
            }
        }
        #post_stream
    };
    stream.into()
}

struct CheckJsonKeys {
    pub keys: Vec<(LitStr, bool)>,
}

impl Parse for CheckJsonKeys {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut keys = Vec::new();
        let first: LitStr = input.parse()?;
        match input.parse::<token::Add>() {
            Ok(_) => {
                keys.push((first, true));
            }
            Err(_) => {
                keys.push((first, false));
            }
        }
        while !input.is_empty() {
            let _: token::Comma = input.parse()?;
            if input.is_empty() {
                break;
            }
            let key: LitStr = input.parse()?;
            match input.parse::<token::Add>() {
                Ok(_) => {
                    keys.push((key, true));
                }
                Err(_) => {
                    keys.push((key, false));
                }
            }
        }
        Ok(Self { keys })
    }
}

#[proc_macro]
pub fn check_json_keys(item: TokenStream) -> TokenStream {
    let CheckJsonKeys { keys } = parse_macro_input!(item as CheckJsonKeys);
    let mut streams = Vec::new();
    for (key, check) in keys {
        if check {
            let k = key.value();
            let k = k.to_case(Case::Snake);
            let fun = Ident::new(&k, key.span());
            streams.push(quote!(#key => {
                self.#fun().try_err(format!("{} {}", gettext("The value of the key <key> is missing:").replace("<key>", key).as_str(), self.data))?;
            }));
        } else {
            streams.push(quote!(#key => {}));
        }
    }
    let stream = quote!(
        {
            use crate::ext::try_err::TryErr;
            use crate::gettext;
            self.data.is_object().try_err(format!("{} {}", gettext("Data is not a object:"), self.data))?;
            for (key, _) in self.data.entries() {
                match key {
                    #(#streams)*
                    _ => { Err(format!("{} {}", gettext("Key <key> is handled:").replace("<key>", key).as_str(), self.data))?; }
                }
            }
        }
    );
    stream.into()
}

#[proc_macro_derive(CheckUnkown)]
pub fn derive_check_unknown(item: TokenStream) -> TokenStream {
    let ItemEnum {
        attrs: _,
        vis: _,
        enum_token: _,
        ident,
        generics: _,
        brace_token: _,
        variants,
    } = parse_macro_input!(item as ItemEnum);
    let mut streams = Vec::new();
    for i in variants.iter() {
        let ident = i.ident.clone();
        let s = ident.to_string();
        if s == "Unknown" {
            streams.push(
                quote!(Self::#ident(data) => Err(crate::fanbox::error::FanboxAPIError::from(format!(
                "{} {}",
                crate::gettext("Failed to recognize type:"),
                data
            ))),),
            );
        } else {
            streams.push(
                quote!(Self::#ident(tmp) => crate::fanbox::check::CheckUnkown::check_unknown(tmp),),
            );
        }
    }
    let stream = quote!(
        impl crate::fanbox::check::CheckUnkown for #ident {
            fn check_unknown(&self) -> Result<(), crate::fanbox::error::FanboxAPIError> {
                match self {
                    #(#streams)*
                }
            }
        }
    );
    stream.into()
}
