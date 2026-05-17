`docker volume ls`
`docker volume inspect dx-rpg_saved_data`
linux: `sudo ls /var/lib/docker/volumes/dx-rpg_saved_data/_data`

`docker exec -it <nom_ou_id_du_conteneur_dx-rpg> sh`
`cd /usr/local/app/saved_data`
`docker run --rm -v dx-rpg_saved_data:/from -v /home/monuser/backup_saved_data:/to alpine sh -c "cp -r /from/* /to/"`
# Avec un conteneur temporaire
copier dans le volume dx-rpg_saved_data :
`docker run --rm -v dx-rpg_saved_data:/data -v /home/monuser/backup_saved_data:/backup alpine sh -c "cp /backup/monfichier.json /data/"`

# modifier le contenu
docker run --rm -it -v dx-rpg_saved_data:/data alpine sh
# Dans le shell, tu peux :
cd /data