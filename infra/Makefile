build-mongo:
	docker build -t mongodb-replicaset:latest . 

start-mongo:
	docker run --rm -d -p 27117:27017 --name mongo mongodb-replicaset && \
	sleep 4 && \
	docker exec mongo mongosh --quiet --eval "rs.initiate();" && \
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet

stop-mongo:
	docker stop mongo

create-db-checkpoint:
	mongodump --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" --out=db_checkpoint --quiet

restore-db-checkpoint:
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet