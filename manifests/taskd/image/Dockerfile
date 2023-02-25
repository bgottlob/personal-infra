FROM alpine:3.17.2
RUN apk add --no-cache taskd=1.1.0-r8 taskd-pki=1.1.0-r8

COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

CMD ["taskd", "server"]
