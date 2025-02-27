use reqwest::blocking::Client;
use scraper::{Html, Selector};
use anyhow::{anyhow, bail, Ok, Result};

const URL: &str = "https://knowyourmeme.com/";

pub type Feed = Vec<Article>;

#[derive(Debug, Clone)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub summary: String,
    pub image_url: String,
    pub meme_name: Option<String>
}

pub fn get_newest_meme_article() -> Result<Article> {
    let mut article = get_newest_article()?;
    article.meme_name = get_meme_title(&article.url).ok();
    Ok(article)
}

pub fn get_newest_article() -> Result<Article> {
    let mut feed = get_feed()?;

    if feed.is_empty() {
        bail!("No articles found");
    }

    let article = feed.remove(0);
    Ok(article)
}

/// Get all articles on the front page excluding editorials but including the meme names.
pub fn get_feed_full() -> Result<Feed> {
    let mut feed = get_feed()?;
    for article in &mut feed {
        article.meme_name = get_meme_title(&article.url).ok();
    }

    Ok(feed)
}

/// Get all articles on the front page.
/// Doesn't include editorials and doesn't fill in the meme name.
fn get_feed() -> Result<Feed> {
    let response = Client::new().get(URL).send()?;
    if !response.status().is_success() {
        bail!("Couldn't reach knowyourmeme.com");
    }

    let html = response.text()?;
    let document = Html::parse_document(html.as_str());

    let articles_selector = Selector::parse(r#"#feed_items > [id^="newsfeed_"]"#).unwrap();
    let article_title_selector = Selector::parse(".newsfeed-title").unwrap();
    let article_image_selector = Selector::parse(".newsfeed_photo").unwrap();
    let article_summary_selector = Selector::parse(".summary").unwrap();

    let mut feed = Feed::new();

    // Extract all articles from the homepage
    for article in document.select(&articles_selector) {
        // Skip editorials
        if article.attr("data-type").unwrap() == "Editorial" {
            continue
        }

        let title = article.select(&article_title_selector).next().unwrap();
        let summary = article.select(&article_summary_selector).next().unwrap();
        let image = match article.select(&article_image_selector).next() {
            Some(img) => img,
            None => continue // Skip article if no image is available
        };
        
        let title_text = title.text().collect::<Vec<_>>().concat();
        let summary_text = summary.text().collect::<Vec<_>>().concat();
        let image_url = image.attr("data-src").unwrap().to_string();

        let article_url = title.attr("href").unwrap();

        // Add article entry
        feed.push(Article {
            url: article_url.to_string(),
            title: title_text,
            summary: summary_text,
            image_url,
            meme_name: None
        });
    }
    Ok(feed)
}

/// Get the name of the meme the article is about
fn get_meme_title(article_url: &str) -> Result<String> {
    let response = Client::new().get(URL.to_string() + article_url).send()?;
    if !response.status().is_success() {
        bail!("Couldn't reach {}", article_url);
    }
    let html = response.text()?;
    let document = Html::parse_document(html.as_str());

    let meme_name_selector_desktop = Selector::parse("section.info > h1:nth-child(1)").unwrap();
    let meme_name_selector_mobile = Selector::parse(".entry-title").unwrap();

    let media_meme_name_selector = Selector::parse("#media-title").unwrap();

    println!("Getting meme title from {}", article_url);

    // 
    if article_url.starts_with("/memes/") {
        // Try get meme name assuming is from the desktop page 
        let name_element = document.select(&meme_name_selector_desktop).next();

        // Try the mobile page selector if the desktop selector failed
        let name_element = match name_element {
            Some(element) => element,
            None => document.select(&meme_name_selector_mobile).next()
                .ok_or(anyhow!("Couldn't find meme name from {}", article_url))?
        };

        Ok(name_element.text().collect::<Vec<_>>().concat())
    } else {
        let name_element = document.select(&media_meme_name_selector).next()
            .ok_or(anyhow!("Couldn't find meme name from {}", article_url))?;
        
        Ok(name_element.text().collect::<Vec<_>>().concat())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert!(get_feed().is_ok());
    }
}
