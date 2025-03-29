use crate::helpers::response::helpers_response::HelpersResponse;
use axum::response::IntoResponse;
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
pub struct ServicesUserEmail;

impl ServicesUserEmail {
    pub async fn send_code(email: &str, code: &str) -> impl IntoResponse {
        dotenv().ok();
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

        let text_body = format!(
            "Recuperação de Senha\n\nUtilize o código: {}\n\nSe você não solicitou essa ação, desconsidere este e-mail.",
            code
        );

        let mailtrap_token =
            std::env::var("MAILTRAP_TOKEN_SECRET").expect("MAILTRAP_TOKEN_SECRET must set");
        let mailtrap_api_token =
            std::env::var("MAILTRAP_API_URL").expect("MAILTRAP_API_URL must set");
        let mailtrap_email_sender =
            std::env::var("MAILTRAP_EMAIL_SENDER").expect("MAILTRAP_EMAIL_SENDER must set");

        let payload = json!({
            "from": {"email": mailtrap_email_sender,},
            "to": [{"email": email,}],
            "subject": "Recuperação de Senha",
            "text": text_body,
            "html": html_body,
        });

        let client = Client::new();
        let response = client
            .post(mailtrap_api_token)
            .header("Content-Type", "application/json")
            .bearer_auth(mailtrap_token)
            .body(payload.to_string())
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                (HelpersResponse::success("E-mail enviado com sucesso!", ""),).into_response()
            }
            Ok(_) => (HelpersResponse::error("Falha ao enviar o e-mail"),).into_response(),
            Err(_) => (HelpersResponse::error(
                "Erro ao conectar ao serviço de e-mail",
            ),)
                .into_response(),
        }
    }
}
