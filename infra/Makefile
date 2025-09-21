docker-db:
	docker build -t employees-manager-database:latest -f Dockerfile-database . 

start-mongo:
	docker run --rm -d -p 27117:27017 --name mongo employees-manager-database && \
	sleep 4 && \
	docker exec mongo mongosh --quiet --eval "rs.initiate();" && \
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet

stop-mongo:
	docker stop mongo

create-db-checkpoint:
	mongodump --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" --out=db_checkpoint --quiet

restore-db-checkpoint:
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet

docker-fe:
	docker build -t employees-manager-frontend:latest --build-arg BUILD_ENVIRONMENT=dev -f Dockerfile-frontend ../employees-manager-fe

docker-be:
	docker build -t employees-manager-backend:latest -f Dockerfile-backend ..


setup-mongo-docker-compose:
	docker exec em-database mongosh --quiet --eval "rs.initiate();" && \
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet