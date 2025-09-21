FROM mongo:7.0.14
CMD [ "mongod", "--replSet=rs0" ]

# 1. build the image
# > docker build -f infra/Dockerfile -t mongodb-replicaset:latest .
# 2. run the container
# > docker run --rm -d -p 27117:27017 --name mongo mongodb-replicaset
# 3. connect to the container
# > docker exec -it mongo /bin/bash
# 4. start mongosh
# > mongosh
# 5. initiate the replica set
# > rs.initiate()
# 6. check the replica set status
# > rs.status()
# connect with compass with the following connection string
# > mongodb://localhost:27117/?replicaSet=rs0&directConnection=true