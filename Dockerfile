# Etapa 1: Build da aplicação
FROM rust:1.84.0 AS builder
WORKDIR /app

# Copia os arquivos do projeto para o contêiner
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Etapa 2: Criação da imagem final com o binário compilado
FROM debian:bookworm-slim

# Instala as dependências necessárias, incluindo libssl
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

# Copia o binário compilado da etapa anterior
COPY --from=builder /app/target/release/blog-back /usr/local/bin/blog-back

# Define o comando padrão para rodar sua aplicação
CMD ["blog-back"]
