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
    server_name 83.228.245.219;

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
