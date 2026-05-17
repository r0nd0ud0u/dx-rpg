# Deploying dx-rpg on a VPS (Infomaniak)

This guide walks you through deploying dx-rpg on a VPS with a real domain name, HTTPS, and Docker Compose.

---

## Overview

```
Internet
  │  HTTPS (443)
  ▼
Nginx  ──── terminates SSL, reverse-proxies to Docker
  │  HTTP (8080) — internal only
  ▼
dx-rpg container
  │
  ├── /data/db.sqlite        (Docker volume — persistent)
  └── /usr/local/app/saved_data/  (Docker volume — persistent)
```

---

## 1. Initial server setup

```bash
# Update system
apt update && apt upgrade -y

# Install Docker
apt install docker.io docker-compose-plugin -y
systemctl enable --now docker

# Install Nginx + Certbot
apt install nginx certbot python3-certbot-nginx -y
systemctl enable --now nginx
```

---

## 2. DNS configuration (Infomaniak)

In your Infomaniak domain manager (Manager → Domains → your domain → DNS zone):

Add an **A record** pointing your domain to your VPS IP:

| Type | Name | Value | TTL |
|------|------|-------|-----|
| A | `@` (or `yourdomain.com`) | `YOUR_VPS_IP` | 300 |
| A | `www` | `YOUR_VPS_IP` | 300 |

> DNS propagation can take 5–30 minutes. Test with: `dig yourdomain.com +short`

---

## 3. Install and start the Docker Compose stack

```bash
# Clone or copy the project on the server
cd /opt
git clone git@github.com:r0nd0ud0u/dx-rpg.git
cd dx-rpg

# Optional: create a .env for runtime secrets (session keys, etc.)
# This file is NOT required if you have no extra secrets.
touch .env

# Pull and start in detached mode
docker compose up -d

# Verify the app responds locally
curl -I http://localhost:8080
```

---

## 4. Configure Nginx with your domain

Create `/etc/nginx/sites-available/dx-rpg`:

```nginx
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;

    # Required by Certbot for the ACME challenge
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # Temporary HTTP proxy before SSL is set up
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket upgrade for /api/ routes
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;  # keep WS connections alive
    }

    location /db/ {
        proxy_pass http://127.0.0.1:8082/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```
delete AAAA on infomaniak
sudo certbot --nginx -d YOUR-DOMAIN.com -d www.YOUR-DOMAIN.com

sudo mkdir -p /var/www/certbot
sudo chown www-data:www-data /var/www/certbot

auto-signed
```
# Redirection HTTP → HTTPS
server {
    listen 80;
    server_name YOUR_IP;
    return 301 https://$host$request_uri;
}

# Configuration HTTPS avec WebSockets
server {
    listen 443 ssl;
    server_name YOUR_IP;

    # Certificat auto-signé
    ssl_certificate /etc/ssl/certs/nginx-selfsigned.crt;
    ssl_certificate_key /etc/ssl/private/nginx-selfsigned.key;

    # Paramètres SSL recommandés
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';

    # Reverse proxy pour l'application principale
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Configuration spécifique pour les WebSockets (ex: /api/new-event)
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Paramètres critiques pour les WebSockets
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location /db/ {
        proxy_pass http://127.0.0.1:8082/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

```

Enable the site and reload:

```bash
ln -s /etc/nginx/sites-available/dx-rpg /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
```

---

## 5. Obtain a free Let's Encrypt SSL certificate

> **Why Let's Encrypt?** It is a free, trusted Certificate Authority (CA). Browsers trust it natively — no "Not Secure" warning, no self-signed certificate prompt.

```bash
certbot --nginx -d yourdomain.com -d www.yourdomain.com
```

Certbot will:
1. Verify domain ownership via the ACME HTTP-01 challenge
2. Download and install the certificate automatically
3. **Rewrite your Nginx config** to add the `listen 443 ssl` block and HTTP→HTTPS redirect

After it finishes, your Nginx config will look like:

```nginx
# Redirect HTTP → HTTPS (added by Certbot)
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name yourdomain.com www.yourdomain.com;

    ssl_certificate     /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    include             /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam         /etc/letsencrypt/ssl-dhparams.pem;

    # Reverse proxy → app
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket upgrade for /api/ routes
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }

    location /db/ {
        proxy_pass http://127.0.0.1:8082/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Test it:
```bash
nginx -t && systemctl reload nginx
curl -I https://yourdomain.com
```

```
# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name your-domain www.your-domain;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name your-domain www.your-domain;

    ssl_certificate     /etc/letsencrypt/live/your-domain/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain/privkey.pem;
    include             /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam         /etc/letsencrypt/ssl-dhparams.pem;

    # Main app
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }

    # sqlite-web under /db/
    location ^~ /db/ {
        auth_basic "Restricted";
        auth_basic_user_file /etc/nginx/.htpasswd;

        proxy_pass http://127.0.0.1:8082/;  # Trailing slash is important!
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        proxy_redirect off;
        proxy_set_header Accept-Encoding "";  # Required for sub_filter
        sub_filter_types text/html;
        sub_filter 'href="/' 'href="/db/';
        sub_filter 'src="/' 'src="/db/';
        sub_filter_once off;
    }

    # Optional: static assets for sqlite-web
    location ^~ /db/static/ {
        proxy_pass http://127.0.0.1:8082/static/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        expires 7d;
        add_header Cache-Control "public, no-transform";
    }

    # Main app catch-all (should be last)
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

```

### Certificate auto-renewal

Certbot installs a systemd timer that renews certificates automatically before they expire (90-day validity).

Check renewal status:
```bash
systemctl status certbot.timer
certbot renew --dry-run   # simulate renewal
```

---

## 6. Managing the Docker stack on the server

```
sudo apt-get update
sudo apt-get install -y ca-certificates curl gnupg
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch=\"$(dpkg --print-architecture)\" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/slinux/ubuntu \
  $(lsb_release -cs) stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
```

```bash
# View running services
docker compose ps

# Follow application logs
docker compose logs -f dx-rpg

# Pull a new image version and restart
docker compose pull && docker compose up -d

# Stop without losing data
docker compose down

# DANGER — removes ALL data (db + saved games)
docker compose down -v
```

### Accessing the SQLite web UI remotely

The `sqlite-web` service is bound to **loopback only** (port 8082) and must NOT be exposed via Nginx. Access it via SSH port forwarding:

```bash
ssh -L 8082:localhost:8082 user@yourdomain.com
# Then open http://localhost:8082 in your browser
```

---

## 7. Firewall (recommended)

```bash
# Allow only SSH, HTTP, HTTPS — block everything else
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
ufw status
```

Port 8080 and 8082 are **not** exposed to the internet. Docker Compose handles internal routing.

---

## Summary checklist

- [ ] DNS A record pointing to VPS IP
- [ ] `docker compose up -d` running
- [ ] Nginx config in `/etc/nginx/sites-available/dx-rpg` and enabled
- [ ] Certbot SSL certificate obtained (`certbot --nginx`)
- [ ] Firewall allows only 22, 80, 443
- [ ] `certbot renew --dry-run` succeeds
- [ ] `https://yourdomain.com` opens without any security warning


## login db

sudo apt-get update
sudo apt-get install apache2-utils  # si ce n'est pas déjà fait
htpasswd -c ./deploy/.htpasswd admin
# (remplace "admin" par le login souhaité, entre le mot de passe quand demandé)