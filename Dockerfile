FROM ekidd/rust-musl-builder:nightly-2019-09-05-openssl11 AS build

WORKDIR /usr/src/aemterliste_rs
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
RUN sudo chown -R rust:rust .
RUN cargo build --release
WORKDIR /usr/src/aemterliste_rs/deployment
RUN sudo chown -R rust:rust .
RUN ls -la /usr/src/aemterliste_rs/target/x86_64-unknown-linux-musl/release && mkdir ./data && touch ./data/aemtermails.txt  && touch ./data/mailmanmails.txt  && touch ./data/mails.txt && mkdir ./tmp && echo "{}" > ./tmp/aemter.json && echo "{}" > ./tmp/aemter27.json && cp /usr/src/aemterliste_rs/target/x86_64-unknown-linux-musl/release/aemterliste_rs ./ && chmod -R 0777 ./ 

# Copy the statically-linked binary into a scratch container.
FROM scratch
WORKDIR /
COPY --from=build /usr/src/aemterliste_rs/deployment /
# Add in certs
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

EXPOSE 8000
ENV SSL_CERT_DIR /etc/ssl/certs
CMD ["./aemterliste_rs"]
