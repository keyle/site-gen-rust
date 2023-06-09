use chrono::Local;
use pulldown_cmark::{html, Options, Parser};
use scraper::{Html, Selector};
use std::fs;

use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct Post {
    pub path: String,
    pub folder: String,
    pub markdown: String,
    pub html: String,
    pub is_blog: bool,
    pub title: String,
    pub url: String,
    pub vanity: String,
    pub pub_date: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl Post {
    pub fn markdown_to_html(&mut self) {
        let contents = fs::read_to_string(self.path.clone()).expect("unable to read file");
        self.markdown = contents.clone();
        // convert contents to html
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&contents, options);
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);
        html_out = html_out.replace(" align=\"left\"", "");
        // html
        self.html = html_out;
    }

    pub fn mangle_template(&mut self, settings: &Settings) {
        let is_blog_post = self.html.contains("<x-blog-title>");
        let is_index = self.html.contains("<x-index/>");
        let html = Html::parse_document(&self.html);
        let title_tag_name;

        let template: String = if is_index {
            fs::read_to_string(settings.templateindex.clone())
                .expect("could not load index template!")
        } else {
            fs::read_to_string(settings.template.clone()).expect("could not load template!")
        };

        let mut contents = template.clone();
        if is_blog_post {
            title_tag_name = "x-blog-title";
            contents = contents.replace("<body>", "<body class='blog'>"); // apply different css
            let x_date = Selector::parse("sub")
                .expect("ERROR Could not extract <sub> (pubdate) from supposed blog post");
            let pubdate = html.select(&x_date).next().unwrap().inner_html(); // TODO impl pubdate in RSS and index page
            self.pub_date = pubdate;
            self.is_blog = true;
        } else {
            title_tag_name = "x-title";

            self.pub_date = Local::now().format("%Y-%m-%d").to_string();
        }
        let x_title = Selector::parse(title_tag_name)
            .expect("ERROR Could not extract <x-title> from supposed BLOG post");
        let title = html.select(&x_title).next().unwrap().inner_html();

        let x_tags = Selector::parse("x-tags")
            .expect("ERROR Could not extract <x-tags> from supposed blog post");
        let tags = html.select(&x_tags).next().unwrap().inner_html();

        self.title = title;

        self.tags = tags
            .split(',')
            .map(|x| x.to_string().trim().to_string())
            .collect();

        if self.html.contains("x-desc") {
            let x_desc = Selector::parse("x-desc")
                .expect("ERROR Could not extract x-desc from supposed PRODUCT post");
            self.description = html.select(&x_desc).next().unwrap().inner_html();
        }

        let vanity = self
            .folder
            .split('/')
            .last()
            .expect("ERROR Could not extract vanity url from folder");
        // NOTE this may change in the future
        self.url = if vanity == "public" {
            self.vanity = String::from("/");
            settings.webroot.clone() + "/" // main index
        } else {
            self.vanity = format!("/blog/posts/{}", &vanity);
            format!("{}/blog/posts/{}", &settings.webroot, &vanity)
        };

        contents = contents.replace(&settings.titletag, &self.title);
        contents = contents.replace(&settings.keywordstag, &tags);
        contents = contents.replace(&settings.descriptiontag, &self.description);
        contents = contents.replace(&settings.contenttag, &self.html);

        self.html = contents;
    }

    pub fn save_html(&mut self) {
        let file_path = format!("{}/index.html", &self.folder);
        fs::write(file_path, &self.html).expect("could not write to html file");
    }
}
