build-mongo:
	docker build -t mongodb-replicaset:latest . 

start-mongo:
	docker run --rm -d -p 27117:27017 --name mongo mongodb-replicaset && \
	sleep 4 && \
	docker exec mongo mongosh --quiet --eval "rs.initiate();"
