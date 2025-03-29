use crate::helpers::response::helpers_response::HelpersResponse;
use axum::response::IntoResponse;
use lettre::message::{Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
pub struct ServicesUserEmail;

impl ServicesUserEmail {
    pub async fn send_code(email: &str, code: &str) -> impl IntoResponse {
        // Cria o corpo do e-mail em HTML com um design personalizado
        let html_body = format!(
            r#"
            <html>
                <head>
                    <style>
                        body {{
                            font-family: Arial, sans-serif;
                            background-color: #f2f2f2;
                            padding: 20px;
                        }}
                        .container {{
                            background-color: #ffffff;
                            padding: 20px;
                            border-radius: 8px;
                            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
                        }}
                        .code {{
                            font-size: 24px;
                            font-weight: bold;
                            color: #333;
                        }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>Recuperação de Senha</h1>
                        <p>Utilize o código abaixo para redefinir sua senha:</p>
                        <p class="code">{}</p>
                        <p>Se você não solicitou essa ação, desconsidere este e-mail.</p>
                    </div>
                </body>
            </html>
            "#,
            code
        );

        // Também é bom enviar um fallback em texto puro
        let text_body = format!(
            "Recuperação de Senha\n\nUtilize o código: {}\n\nSe você não solicitou essa ação, desconsidere este e-mail.",
            code
        );

        // Cria a mensagem de e-mail com corpo multipart (plain text e HTML)
        let email_message = Message::builder()
            .from("Seu Nome <fa927837e892ce@mailtrap.io>".parse().unwrap())
            .to(format!("<{}>", email).parse().unwrap())
            .subject("Recuperação de Senha")
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(text_body))
                    .singlepart(SinglePart::html(html_body)),
            )
            .unwrap();
        println!("email_message {:?}", email_message);
        // Configura as credenciais e o transporte SMTP
        let creds = Credentials::new("fa927837e892ce".to_string(), "8ddb12ce08a25d".to_string());
        println!("creds {:?}", creds);

        let mailer = SmtpTransport::relay("sandbox.smtp.mailtrap.io")
            .unwrap()
            .port(2525)
            .credentials(creds)
            .build();

        println!("mailer {:?}", mailer);

        // Tenta enviar o e-mail
        match mailer.send(&email_message) {
            Ok(_) => (HelpersResponse::success("E-mail enviado com sucesso!", ""),).into_response(),
            Err(e) => {
                eprintln!("Erro ao enviar e-mail: {:?}", e);
                (HelpersResponse::error("Falha ao enviar o e-mail")).into_response()
            }
        }
    }
}
