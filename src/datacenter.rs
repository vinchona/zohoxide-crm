#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DataCenter {
    US,
    AU,
    EU,
    IN,
    CN,
    JP,
    CA
}

impl DataCenter {
    pub fn get_iam_url(&self) -> String {
        match self {
            DataCenter::US => String::from("https://accounts.zoho.com"),
            DataCenter::AU => String::from("https://accounts.zoho.com.au"),
            DataCenter::EU => String::from("https://accounts.zoho.eu"),
            DataCenter::IN => String::from("https://accounts.zoho.in"),
            DataCenter::CN => String::from("https://accounts.zoho.com.cn"),
            DataCenter::JP => String::from("https://accounts.zoho.jp"),
            DataCenter::CA => String::from("https://accounts.zoho.ca"),
        }
    }

    pub fn get_file_upload_url(&self) -> String {
        match self {
            DataCenter::US => String::from("https://content.zohoapis.com"),
            DataCenter::AU => String::from("https://content.zohoapis.com.au"),
            DataCenter::EU => String::from("https://content.zohoapis.eu"),
            DataCenter::IN => String::from("https://content.zohoapis.in"),
            DataCenter::CN => String::from("https://content.zohoapis.com.cn"),
            DataCenter::JP => String::from("https://content.zohoapis.jp"),
            DataCenter::CA => String::from("https://content.zohoapis.ca"),
        }
    }
}
