use convert_case::Case;
use convert_case::Casing;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::io::Read;
use std::str::FromStr;
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
                            use crate::fanbox::check::CheckUnknown;
                            println!("{:#?}", data);
                            match data.check_unknown() {
                                Ok(_) => {}
                                Err(e) => {
                                    panic!("Check unknown: {}", e);
                                }
                            }
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
            if input.cursor().eof() {
                break;
            }
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
    let mut options_method = None;
    for method in methods {
        header_value.push(method.to_string());
        if method == "OPTIONS" && handle_options.value() {
            enable_options = true;
            options_method = Some(method);
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
        let options_method = options_method.unwrap();
        streams.push(quote!(&hyper::Method::#options_method => {
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

type CheckJsonKey = (LitStr, bool, Option<Ident>, Option<CheckJsonKeys>);

struct CheckJsonKeys {
    pub keys: Vec<CheckJsonKey>,
}

impl Parse for CheckJsonKeys {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut keys = Vec::new();
        if input.cursor().eof() {
            return Ok(Self { keys });
        }
        let first: LitStr = input.parse()?;
        match input.parse::<token::Add>() {
            Ok(_) => match input.parse::<Ident>() {
                Ok(ident) => {
                    keys.push((first, true, Some(ident), None));
                }
                Err(_) => {
                    keys.push((first, true, None, None));
                }
            },
            Err(_) => match input.parse::<token::Colon>() {
                Ok(_) => {
                    let content;
                    bracketed!(content in input);
                    let childrens = CheckJsonKeys::parse(&content)?;
                    keys.push((first, false, None, Some(childrens)));
                }
                Err(_) => {
                    keys.push((first, false, None, None));
                }
            },
        }
        while !input.is_empty() {
            let _: token::Comma = input.parse()?;
            if input.is_empty() {
                break;
            }
            let key: LitStr = input.parse()?;
            match input.parse::<token::Add>() {
                Ok(_) => match input.parse::<Ident>() {
                    Ok(ident) => {
                        keys.push((key, true, Some(ident), None));
                    }
                    Err(_) => {
                        keys.push((key, true, None, None));
                    }
                },
                Err(_) => match input.parse::<token::Colon>() {
                    Ok(_) => {
                        let content;
                        bracketed!(content in input);
                        let childrens = CheckJsonKeys::parse(&content)?;
                        keys.push((key, false, None, Some(childrens)));
                    }
                    Err(_) => {
                        keys.push((key, false, None, None));
                    }
                },
            }
        }
        Ok(Self { keys })
    }
}

fn get_check_json_keys_streams(
    keys: Vec<CheckJsonKey>,
    pkeys: Vec<LitStr>,
) -> Vec<proc_macro2::TokenStream> {
    let mut streams = Vec::new();
    for (key, check, fname, childrens) in keys {
        match childrens {
            Some(childrens) => {
                let mut keys2 = pkeys.clone();
                keys2.push(key.clone());
                let streams2 = get_check_json_keys_streams(childrens.keys, keys2);
                streams.push(quote!(#key => {
                    let obj = sobj;
                    obj.is_object().try_err(format!("{} {}", gettext("Data is not a object:"), obj))?;
                    for (key, sobj) in obj.entries() {
                        match key {
                            #(#streams2)*
                            _ => { Err(format!("{} {}", gettext("Key <key> is not handled:").replace("<key>", key).as_str(), obj))?; }
                        }
                    }
                }))
            }
            None => {
                if check {
                    let mut keys = Vec::new();
                    for i in pkeys.iter() {
                        keys.push(i.value());
                    }
                    keys.push(key.value());
                    let k = keys.join("_").to_case(Case::Snake);
                    let fun = match fname {
                        Some(fname) => fname,
                        None => Ident::new(&k, key.span()),
                    };
                    streams.push(quote!(#key => {
                        self.#fun().try_err(format!("{} {}", gettext("The value of the key <key> is missing:").replace("<key>", key).as_str(), obj))?;
                    }));
                } else {
                    streams.push(quote!(#key => {}));
                }
            }
        }
    }
    streams
}

#[proc_macro]
pub fn check_json_keys(item: TokenStream) -> TokenStream {
    let CheckJsonKeys { keys } = parse_macro_input!(item as CheckJsonKeys);
    let streams = get_check_json_keys_streams(keys, Vec::new());
    let stream = quote!(
        {
            use crate::ext::try_err::TryErr;
            use crate::gettext;
            self.data.is_object().try_err(format!("{} {}", gettext("Data is not a object:"), self.data))?;
            let obj = &self.data;
            for (key, sobj) in self.data.entries() {
                match key {
                    #(#streams)*
                    _ => { Err(format!("{} {}", gettext("Key <key> is not handled:").replace("<key>", key).as_str(), obj))?; }
                }
            }
        }
    );
    stream.into()
}

#[proc_macro_derive(CheckUnknown)]
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
                quote!(Self::#ident(tmp) => crate::fanbox::check::CheckUnknown::check_unknown(tmp),),
            );
        }
    }
    let stream = quote!(
        impl crate::fanbox::check::CheckUnknown for #ident {
            fn check_unknown(&self) -> Result<(), crate::fanbox::error::FanboxAPIError> {
                match self {
                    #(#streams)*
                }
            }
        }
    );
    stream.into()
}

struct CreateFanboxDownloadHelper {
    pub fn_name: Ident,
}

impl Parse for CreateFanboxDownloadHelper {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fn_name = input.parse()?;
        Ok(Self { fn_name })
    }
}

#[proc_macro]
pub fn create_fanbox_download_helper(item: TokenStream) -> TokenStream {
    let CreateFanboxDownloadHelper { fn_name } =
        parse_macro_input!(item as CreateFanboxDownloadHelper);
    let defn_name = Ident::new(
        format!("download_{}", fn_name.to_string()).as_str(),
        fn_name.span(),
    );
    let stream = quote!(
        #[inline]
        pub fn #defn_name(&self) -> Result<Option<crate::downloader::DownloaderHelper>, crate::downloader::DownloaderError> {
            match self.#fn_name() {
                Some(url) => {
                    let client: &std::sync::Arc<crate::webclient::WebClient> = self.client.as_ref().as_ref();
                    Ok(Some(crate::downloader::DownloaderHelper::builder(url)?.client(client).build()))
                }
                None => Ok(None),
            }
        }
    );
    stream.into()
}

fn read_json_file<P: AsRef<std::path::Path> + ?Sized>(p: &P) -> json::JsonValue {
    let mut f = std::fs::File::open(p).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    json::parse(s.as_str()).unwrap()
}

#[proc_macro]
pub fn define_exif_data_source(item: TokenStream) -> TokenStream {
    let file_path = parse_macro_input!(item as LitStr).value();
    let path =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(file_path);
    let data = read_json_file(&path);
    let mut streams = Vec::new();
    for (key, data) in data.entries() {
        let doc = match data["description"].as_str() {
            Some(doc) => {
                let doc = LitStr::new(doc, Span::call_site());
                quote!(#[doc = #doc])
            }
            None => {
                quote!()
            }
        };
        let fname = Ident::new(key, Span::call_site());
        let result = proc_macro2::TokenStream::from_str(match data["return"].as_str() {
            Some(r) => r,
            None => "String",
        })
        .unwrap();
        streams.push(quote!(
            #doc
            #[inline]
            fn #fname(&self) -> Option<#result> {
                None
            }
        ));
    }
    let stream = quote!(
        #(#streams)*
    );
    stream.into()
}

struct CallParentDataSourceFun {
    pub file: LitStr,
    pub expr: Expr,
    pub keys: Vec<Ident>,
    pub is_include: bool,
}

impl Parse for CallParentDataSourceFun {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let filel = Lit::parse(input)?;
        let file = match filel {
            Lit::Str(s) => s,
            _ => {
                return Err(syn::Error::new(filel.span(), "Unsupported literal."));
            }
        };
        token::Comma::parse(input)?;
        let expr = input.parse()?;
        let mut keys = Vec::new();
        let mut is_include = false;
        let mut is_first = true;
        loop {
            if input.cursor().eof() {
                break;
            }
            token::Comma::parse(input)?;
            if input.cursor().eof() {
                break;
            }
            if is_first {
                is_first = false;
                match syn::Lit::parse(input) {
                    Ok(lit) => {
                        match lit {
                            syn::Lit::Bool(lit) => {
                                is_include = lit.value();
                            }
                            _ => {
                                return Err(syn::Error::new(lit.span(), "Unsupported literal."));
                            }
                        }
                        continue;
                    }
                    Err(_) => {}
                }
            }
            keys.push(Ident::parse(input)?);
        }
        Ok(Self {
            file,
            expr,
            keys,
            is_include,
        })
    }
}

#[proc_macro]
pub fn call_parent_data_source_fun(item: TokenStream) -> TokenStream {
    let CallParentDataSourceFun {
        file,
        expr,
        keys,
        is_include,
    } = parse_macro_input!(item as CallParentDataSourceFun);
    let path =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(file.value());
    let data = read_json_file(&path);
    let mut streams = Vec::new();
    for (key, data) in data.entries() {
        let is_in = keys.iter().find(|&r| r.to_string() == key).is_some();
        if is_in != is_include {
            continue;
        }
        let fname = Ident::new(key, Span::call_site());
        let result = proc_macro2::TokenStream::from_str(match data["return"].as_str() {
            Some(r) => r,
            None => "String",
        })
        .unwrap();
        streams.push(quote!(
            #[inline]
            fn #fname(&self) -> Option<#result> {
                (#expr).#fname()
            }
        ));
    }
    let stream = quote!(
        #(#streams)*
    );
    stream.into()
}
