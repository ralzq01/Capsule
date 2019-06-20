extern crate serde_json;
extern crate lettre;
extern crate native_tls;

use std::collections::HashMap;
use lettre_email::EmailBuilder;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{ClientSecurity, ClientTlsParameters, SendableEmail, 
  EmailAddress, Envelope, Transport, SmtpClient,
};
use lettre::smtp::ConnectionReuseParameters;
use native_tls::{Protocol, TlsConnector};
use serde_json::Value;

use crate::doer::base::MyDoer;
use crate::doer::base::Status;


pub struct EmailSender<'a> {
  sender: &'a str,
  password: &'a str,
  recver: &'a str,
  subject: &'a str,
  content: &'a str,
}

impl<'a> EmailSender<'a> {
  pub fn new(config: &'a HashMap<String, String>) -> EmailSender {
    let sender = config.get("sender").expect(
      "Lack of \"sender\" field in EmailSender."
    );
    let password = config.get("password").expect(
      "Lack of \"password\" field in EmailSender."
    );
    let recver = config.get("recver").expect(
      "Lack of \"recver\" field in EmailSender."
    );
    let subject = config.get("subject").expect(
      "Lack of \"subject\" field in EmailSender."
    );
    let content = config.get("content").expect(
      "Lack of \"content\" field in EmailSender."
    );
    EmailSender {
      sender,
      password,
      recver,
      subject,
      content,
    }
  }

  fn replace_content<'b> (&self, c: &'b str) -> String {
    // TODO: implement replacement
    self.content.to_string()
  }
}

impl<'a> MyDoer for EmailSender<'a> {
  /// recv a json string from upper watcher
  /// replace the self.content: `{}` to the string
  fn get(&self, input: String) -> Status {
    let event: Value = serde_json::from_str(&input).unwrap();
    let content = self.replace_content(input.as_str());
    let email = EmailBuilder::new()
      .from(self.sender)
      .to(self.recver)
      .subject(self.subject)
      .text(content.as_str())
      .build()
      .expect("Failed to build message");

    // create stmp connection
    let creds = Credentials::new(
      self.sender.to_string(),
      self.password.to_string(),
    );
    let addr = "smtp.office365.com:587";
    let connector = TlsConnector::new().unwrap();
    let tls_params = ClientTlsParameters::new(String::from("smtp.office365.com"), connector);
    let security = ClientSecurity::Required(tls_params);
    let mut mailer = SmtpClient::new(addr, security)
      .expect("Error creating Smtp client")
      .credentials(creds)
      .transport();

    // send mail
    let result = mailer.send(email.into());
    println!("{:?}", result);
    assert!(result.is_ok());
    mailer.close();
    // need work
    Status::OkNone
  }

}


