#!/usr/bin/env sh
set -e
PKI="${TASKDDATA}/pki"

if [ ! -d "${PKI}" ]; then
    echo "${PKI} does not exist, creating it now"
    mkdir -p "${TASKDDATA}"
    cp -r /usr/share/taskd/pki "${PKI}"
    taskd init > /dev/null 2>&1
else
    echo "${PKI} already exists, skipping"
fi

if [ ! -f "${PKI}/ca.cert.pem" ]; then
    echo "Certs have not yet been generated, generating them now"
    cd "${PKI}"

    sed -i "/^CN=.*$/d" vars
    sed -i "/^ORGANIZATION=.*$/d" vars
    sed -i "/^COUNTRY=.*$/d" vars
    sed -i "/^STATE=.*$/d" vars
    sed -i "/^LOCALITY=.*$/d" vars

    cat >> vars <<EOF
CN=${CN}
ORGANIZATION=${CERT_ORGANIZATION}
COUNTRY=${CERT_COUNTRY}
STATE=${CERT_STATE}
LOCALITY=${CERT_LOCALITY}
EOF

    ./generate
    taskd config --force client.cert "${PKI}/client.cert.pem" > /dev/null 2>&1
    taskd config --force client.key "${PKI}/client.key.pem" > /dev/null 2>&1
    taskd config --force server.cert "${PKI}/server.cert.pem" > /dev/null 2>&1
    taskd config --force server.key "${PKI}/server.key.pem" > /dev/null 2>&1
    taskd config --force server.crl "${PKI}/server.crl.pem" > /dev/null 2>&1
    taskd config --force ca.cert "${PKI}/ca.cert.pem" > /dev/null 2>&1

    taskd add org "${USER_ORG:=Public}" > /dev/null 2>&1
    taskd add user "${USER_ORG}" "${USER_FIRST_NAME} ${USER_LAST_NAME}"
    ./generate.client "${USER_FIRST_NAME}_${USER_LAST_NAME}"
else
    echo "Certs have already been generated, skipping"
fi

# Sets logs to go to STDOUT
taskd config --force log "-" > /dev/null 2>&1
taskd config --force server "0.0.0.0:53589" > /dev/null 2>&1

exec "${@}"
