use derive_builder::Builder;
use derive_setters::Setters;
use reqwest::header::HeaderMap;
use reqwest::multipart::{Form, Part};
use std::path::{Path, PathBuf};

#[derive(Debug, derive_more::From)]
pub enum FormDataBody {
    Data(Vec<u8>),
    File(PathBuf),
}

#[derive(Builder, Debug, Setters)]
#[builder(pattern = "owned", setter(strip_option))]
#[setters(borrow_self, into)]
/// Part of form
pub struct FormDataPart {
    #[builder(default, setter(into))]
    #[setters(strip_option)]
    /// Mime type
    mime: Option<String>,
    #[builder(default, setter(into))]
    #[setters(strip_option)]
    /// File name
    filename: Option<String>,
    #[builder(default)]
    #[setters(skip)]
    /// HTTP headers
    pub headers: HeaderMap,
    #[setters(skip)]
    #[builder(setter(into))]
    /// Body
    body: FormDataBody,
}

/// Form
pub struct FormData {
    fields: Vec<(String, FormDataPart)>,
}

/// Error when convert [FormData] to [Form]
#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum FormDataError {
    IOError(std::io::Error),
    Reqwest(reqwest::Error),
}

impl FormData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data<'a, K: AsRef<str> + ?Sized, V: AsRef<[u8]> + ?Sized>(
        &'a mut self,
        key: &K,
        value: &V,
    ) -> &'a mut FormDataPart {
        let part = FormDataPartBuilder::default()
            .body(FormDataBody::Data(value.as_ref().to_vec()))
            .build()
            .unwrap();
        self.fields.push((key.as_ref().to_owned(), part));
        &mut self.fields.last_mut().unwrap().1
    }

    pub fn file<'a, K: AsRef<str> + ?Sized, P: AsRef<Path> + ?Sized>(
        &'a mut self,
        key: &K,
        path: &P,
    ) -> &'a mut FormDataPart {
        let part = FormDataPartBuilder::default()
            .body(FormDataBody::File(path.as_ref().to_owned()))
            .build()
            .unwrap();
        self.fields.push((key.as_ref().to_owned(), part));
        &mut self.fields.last_mut().unwrap().1
    }

    pub fn part<'a, K: AsRef<str> + ?Sized>(
        &'a mut self,
        key: &K,
        part: FormDataPart,
    ) -> &'a mut FormDataPart {
        self.fields.push((key.as_ref().to_owned(), part));
        &mut self.fields.last_mut().unwrap().1
    }

    pub async fn to_form(&self) -> Result<Form, FormDataError> {
        let mut f = Form::new();
        for (k, v) in self.fields.iter() {
            let mut part = match &v.body {
                FormDataBody::Data(d) => Part::bytes(d.clone()),
                FormDataBody::File(f) => Part::bytes(tokio::fs::read(f).await?),
            };
            match &v.mime {
                Some(m) => {
                    part = part.mime_str(m)?;
                }
                None => {}
            }
            match &v.filename {
                Some(f) => {
                    part = part.file_name(f.clone());
                }
                None => {}
            }
            part = part.headers(v.headers.clone());
            f = f.part(k.clone(), part);
        }
        Ok(f)
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self { fields: Vec::new() }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_formdata() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = std::fs::create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    std::fs::write("test/formdata.txt", "Good job!").unwrap();
    let mut f = FormData::new();
    f.data("test", "test2").filename("test.txt");
    f.file("test2", "test/formdata.txt")
        .filename("formdata.txt")
        .mime("text/plain");
    f.to_form().await.unwrap();
}
