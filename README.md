# ARTEMIS
Blog and wiki framework for [uauth.io](https://uauth.io)

## Github Webhooks
Artemis supports authenticated webhooks for blog and wiki entries.  
Point the webhook to `/update` and set the key in `/etc/artemis/config.yml` + Github.

## Well-known & pgp-key.txt
- Put the files you want to serve underneath the `/.well-known` directory in `/etc/artemis/well-known` (needs to be created).
- Save your public exported key to `/etc/artemis/pgp-key.txt`

Be aware that this is heavily configured to be used by myself and nobody else.  
Feel free to fork and modify if you want to use it for your personal blog/wiki. 
