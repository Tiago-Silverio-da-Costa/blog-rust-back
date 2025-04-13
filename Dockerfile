# Etapa 1: Build da aplicação
FROM rust:1.84.0 AS builder
WORKDIR /app

# Copia os arquivos do projeto para o container
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Etapa 2: Criação da imagem final com o binário compilado
FROM debian:buster-slim

# Copia o binário compilado da etapa anterior
COPY --from=builder /app/target/release/seu_binario /usr/local/bin/seu_binario

# Define o comando padrão para rodar sua aplicação
CMD ["seu_binario"]
