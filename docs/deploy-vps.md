apt install docker.io docker-compose -y
systemctl enable docker
systemctl start docker

apt install nginx certbot python3-certbot-nginx -y

sudo apt install nginx -y
sudo systemctl start nginx
sudo systemctl enable nginx

server {
    listen 80;
    server_name votre-domaine.com www.votre-domaine.com;

    root /var/www/votre-domaine.com/html;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}

sudo ln -s /etc/nginx/sites-available/votre-domaine.com /etc/nginx/sites-enabled/

sudo vim /etc/nginx/sites-available/dx-rpg
server {
    listen 80;
    server_name your-ip;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Configuration spécifique pour les WebSockets
    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Paramètres obligatoires pour les WebSockets
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}

sudo ln -s /etc/nginx/sites-available/dx-rpg /etc/nginx/sites-enabled/


sudo systemctl restart nginx

sudo docker run -d --name dx-rpg -p 8080:8080 ghcr.io/r0nd0ud0u/dx-rpg:latest


## certificat auto signed

`sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048   -keyout /etc/ssl/private/nginx-selfsigned.key   -out /etc/ssl/certs/nginx-selfsigned.crt`

```
# Redirection HTTP → HTTPS
server {
    listen 80;
    server_name your-ip;
    return 301 https://$host$request_uri;
}

# Configuration HTTPS avec WebSockets
server {
    listen 443 ssl;
    server_name your-ip;

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
}
```