FROM rust:1.76
WORKDIR /app
VOLUME /db
COPY . .
RUN cargo install --path .
EXPOSE 8040
CMD ["esbn_plus"]
