use http::{
    headers::HeaderValue,
    headers::{ToHeaderValues, COOKIE},
    Cookie, Method, Request, Response, Url,
};
use tide::{http, Body, Result, Server};

pub struct Session<State> {
    app: Server<State>,
    cookies: Cookies,
    base_url: Url,
}

use scraper::ElementRef;
#[cfg(feature = "html")]
pub use scraper::{Html, Selector};
use serde::{de::DeserializeOwned, Serialize};

impl<State> Session<State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn new(app: Server<State>) -> Self {
        Self {
            app,
            cookies: Cookies::default(),
            base_url: Url::parse("http://example.com/").unwrap(),
        }
    }

    pub fn app(&self) -> &Server<State> {
        &self.app
    }

    pub fn cookies(&self) -> &Cookies {
        &self.cookies
    }

    pub fn set_base_url(&mut self, url: &str) {
        self.base_url = Url::parse(url).unwrap();
    }

    pub fn base_url(&mut self) -> &Url {
        &self.base_url
    }

    pub async fn respond(&mut self, mut request: Request) -> Result<Response> {
        request.append_header(COOKIE, &self.cookies);
        let response: Response = self.app.respond(request).await?;
        self.cookies.update_from_response(&response);
        Ok(response)
    }
    pub async fn get(&mut self, path: &str) -> Result<Response> {
        self.respond(Request::new(Method::Get, self.base_url.join(path).unwrap()))
            .await
    }

    pub async fn get_json<T: DeserializeOwned>(&mut self, path: &str) -> tide::Result<T> {
        let mut response = self.get(path).await?;
        response.body_json::<T>().await
    }

    pub async fn get_string(&mut self, path: &str) -> Result<String> {
        let mut response = self.get(path).await?;
        response.body_string().await
    }

    #[cfg(feature = "html")]
    pub async fn get_html(&mut self, path: &str) -> Result<HtmlDocument> {
        let mut response = self.get(path).await?;
        let string = response.body_string().await?;
        Ok(HtmlDocument(Html::parse_document(&string), string))
    }

    pub async fn post(&mut self, path: &str, body: &impl Serialize) -> Result<Response> {
        let mut request = Request::new(Method::Post, self.base_url.join(path).unwrap());
        request.set_body(Body::from_form(&body)?);
        self.respond(request).await
    }
}

#[derive(Debug, Clone)]
pub struct HtmlDocument(Html, String);

impl HtmlDocument {
    pub fn select_one<'a, 'b>(&'a self, selector_str: &'b str) -> ElementRef<'a> {
        let selector = Selector::parse(selector_str).unwrap();
        let mut selection = self.0.select(&selector);
        selection.next().expect(&format!(
            "expected at least one `{}`, but found none.\n\n *** document *** \n\n{}\n\n *** end document ***\n\n",
            selector_str,
            self.root_element().inner_html()
        ))
    }

    pub fn count_matching<'a, 'b>(&'a self, selector: &'b str) -> usize {
        let selector = Selector::parse(selector).unwrap();
        self.0.select(&selector).count()
    }
}

impl std::ops::DerefMut for HtmlDocument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for HtmlDocument {
    type Target = Html;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cookies(Vec<Cookie<'static>>);
impl Cookies {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn update(&mut self, cookies: Cookies) {
        for cookie in cookies {
            let value = cookie.value();
            if value == "" {
                self.remove(cookie.name());
            } else {
                self.add(cookie);
            }
        }
    }

    pub fn update_from_response(&mut self, response: &Response) {
        self.update(Cookies::from_response(response));
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        self.remove(cookie.name());
        self.0.push(cookie);
    }

    pub fn remove(&mut self, name: &str) {
        self.0 = self
            .0
            .drain(..)
            .filter(|cookie| cookie.name() == name)
            .collect();
    }

    pub fn from_response(response: &Response) -> Self {
        response
            .header("Set-Cookie")
            .map(|hv| hv.to_string())
            .unwrap_or_else(|| "[]".into())
            .parse()
            .unwrap()
    }

    fn get<'a>(&'a self, name: &str) -> Option<&'a Cookie<'static>> {
        self.0.iter().find(|cookie| cookie.name() == name)
    }
}

impl IntoIterator for Cookies {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Cookie<'static>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ToHeaderValues for &Cookies {
    type Iter = std::iter::Once<HeaderValue>;
    fn to_header_values(&self) -> http::Result<Self::Iter> {
        let value = self
            .0
            .iter()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>()
            .join("; ");
        Ok(std::iter::once(HeaderValue::from_bytes(value.into())?))
    }
}

impl std::ops::Index<&str> for Cookies {
    type Output = Cookie<'static>;
    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl std::str::FromStr for Cookies {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let strings: Vec<String> = serde_json::from_str(s).unwrap_or_default();

        Ok(Self(
            strings
                .iter()
                .filter_map(|cookie| Cookie::parse(cookie.to_owned()).ok())
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tide::prelude::*;

    async fn build_app() -> Server<()> {
        let mut app = tide::new();

        app.with(tide::sessions::SessionMiddleware::new(
            tide::sessions::MemoryStore::new(),
            b"0123456789012345678901234567890123",
        ));

        app.at("/").get(|mut req: tide::Request<()>| async move {
            let session = req.session_mut();
            let visits = session.get::<usize>("visits").unwrap_or_default() + 1;
            session.insert("visits", visits).unwrap();
            Ok(tide::Body::from_json(&Visits { visits })?)
        });

        app
    }

    #[derive(Serialize, Deserialize)]
    struct Visits {
        visits: usize,
    }

    #[async_std::test]
    async fn test() -> tide::Result<()> {
        let app = build_app().await;
        let mut session = Session::new(app);
        let response: Visits = session.get_json("/").await.unwrap();
        assert_eq!(response.visits, 1);
        let response: Visits = session.get_json("/").await.unwrap();
        assert_eq!(response.visits, 2);

        Ok(())
    }
}
