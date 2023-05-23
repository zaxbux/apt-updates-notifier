use lettre::{
    message::{header::ContentType, MultiPart},
    Message, SmtpTransport, Transport,
};

use crate::{
    apt::{ProgressOutput, Upgradeable},
    config,
    error::Result,
};

pub fn build_message(
    config: &config::Config,
    upgradeable: &Vec<Upgradeable>,
    output: &Vec<ProgressOutput>,
) -> Result<Message> {
    let message = Message::builder()
        .sender(config.mail.from()?)
        .from(config.mail.from()?)
        .mailbox(config.mail.to_header())
        .subject(config.mail.subject_fmt(upgradeable.len()));

    let message = if config.mail.html() {
        message.multipart(MultiPart::alternative_plain_html(
            make_plain(&config.mail, upgradeable, output),
            make_html(&config.mail, upgradeable, output),
        ))?
    } else {
        message.header(ContentType::TEXT_PLAIN).body(make_plain(
            &config.mail,
            upgradeable,
            output,
        ))?
    };

    Ok(message)
}

pub fn send_smtp(
    config: &config::Config,
    message: Message,
) -> Result<lettre::transport::smtp::response::Response> {
    let mailer = SmtpTransport::relay(&config.smtp.relay())?
        .credentials(config.smtp.credentials())
        .build();

    Ok(mailer.send(&message)?)
}

fn make_plain(
    mail_config: &config::Mail,
    upgradeable: &Vec<Upgradeable>,
    output: &Vec<ProgressOutput>,
) -> String {
    let mut buf: Vec<String> = Vec::new();

    if let Some(prepend) = mail_config.prepend() {
        buf.push(prepend);
    }

    buf.push(String::from("# Packages\n"));

    buf.extend(upgradeable.iter().map(|pkg| {
        format!(
            //"{}@{}\t➭\t{}",
            "{}/{} {} {} [upgradeable from: {}]",
            pkg.name,
			pkg.archive,
			pkg.arch,
            pkg.candidate.clone().unwrap_or("".to_string()),
            pkg.installed.clone().unwrap_or("".to_string()),
        )
    }));

    buf.push(String::from("\n"));
    buf.push(String::from("# Output\n"));
    buf.extend(output.iter().map(|o| format!("{}", o)));

    if let Some(append) = mail_config.append() {
        buf.push(append);
    }

    buf.join("\n")
}

fn make_html(
    mail_config: &config::Mail,
    upgradeable: &Vec<Upgradeable>,
    output: &Vec<ProgressOutput>,
) -> String {
    let mut buf: Vec<String> = Vec::new();

    buf.push(String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
    </head>
    <body>"#,
    ));

    if let Some(prepend) = mail_config.prepend() {
        buf.push(prepend);
    }

    buf.push(String::from(
        r#"        <h1>Packages</h1>
        <table border="1" rules="all">
            <tr>
                <th>Package</th>
                <th>Installed</th>
                <th>Candidate</th>
            </tr>"#,
    ));

    buf.extend(upgradeable.iter().map(|pkg| {
        format!(
            "<tr><td><code>{}</code><br><code>/{} {}</code></td><td><code>{}</code></td><td><code>{}</code></td></tr>",
            pkg.name,
			pkg.archive,
			pkg.arch,
            pkg.installed.clone().unwrap_or("".to_string()),
            pkg.candidate.clone().unwrap_or("".to_string()),
        )
    }));

    buf.push(String::from(r#"        </table><h1>Output</h1><pre>"#));
    buf.extend(output.iter().map(|o| format!("{}", o)));
    buf.push(String::from(r#"</pre>"#));

    if let Some(append) = mail_config.append() {
        buf.push(append);
    }

    buf.push(String::from(
        r#"
    </body>
    </html>"#,
    ));

    buf.join("\n")
}
