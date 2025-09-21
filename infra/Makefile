build-testing-docker-db:
	@# build docker image of database with one replica for testing purpose
	docker build -t employees-manager-database:latest -f Dockerfile-database .

start-testing-docker-db:
	@# start container from testing database image, starts the replica server and restore database checkpoint
	docker run --rm -d -p 27117:27017 --name mongo employees-manager-database && \
	sleep 2 && \
	docker exec mongo mongosh --quiet --eval "rs.initiate();" && \
	sleep 2 && \
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/

stop-testing-docker-db:
	@# stops the running container of the testing database
	docker stop mongo

create-local-db-checkpoint:
	@# dumps local database
	mongodump --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" --out=db_checkpoint --quiet

restore-local-db-checkpoint:
	@# restore the local database with the local dump
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet

setup-mongo-local-docker-compose:
	@# starts replica set and restore database checkpoint
	docker exec em-database mongosh --quiet --eval "rs.initiate();" && \
	mongorestore --drop --uri="mongodb://localhost:27117/?replicaSet=rs0&directConnection=true" --db="application-database-dev" db_checkpoint/application-database-dev/ --quiet

setup-mongo-pre:
	@# restore database dump for pre database
	mongorestore --drop --uri="mongodb+srv://$(DB_USER):$(DB_PWD)@employees-manager-pre.u2l5d.mongodb.net/" --db="application-database-pre" db_checkpoint-pre/application-database-pre/

build-local-docker-fe:
	docker build -t employees-manager-frontend:latest --build-arg BUILD_ENVIRONMENT=dev -f Dockerfile-frontend ../employees-manager-fe

build-local-docker-be:
	docker build -t employees-manager-backend:latest -f Dockerfile-backend ..

build-local-integrated-docker:
	@# build docker image for integrated application without pushing to any registry
	docker build -t employees-manager-app:latest --build-arg BUILD_ENVIRONMENT=dev -f Dockerfile-integrated ..

build-gcp-integrated-docker:
	@# build docker image for integrated application and pushes to GCP registry

	@# login
	gcloud auth print-access-token | docker login -u oauth2accesstoken --password-stdin https://$(GCP_REGISTRY_REGION)-docker.pkg.dev
	$(eval DEPLOY_ENVIRONMENT=$(shell echo echo $(VERSION_TAG) | cut -d '@' -f 2))
	$(eval DOCKER_REPOSITORY=$(GCP_REGISTRY_REGION)-docker.pkg.dev/$(GCP_PROJECT_ID)/employees-manager)
	$(eval IMAGE_NAME=$(DOCKER_REPOSITORY)/employees-manager)

	@# verify that version tag is set
	@[ -z "$(VERSION_TAG)" ] && echo VERSION_TAG is empty && exit 1 || echo "deploying $(VERSION_TAG)"

	@# the tag will have the following structure "vM.m.p@DE" where
	@# M is the major version
	@# m is the minor version
	@# p is the patch version
	@# DE is the deploy environment
	@# therefore, split by @ to get the DE or the versions
	@# then we split by v to remove it
	@# and finally we split by . to get the version level
	$(eval MAJOR=$(shell echo echo $(VERSION_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 1))
	$(eval MINOR=$(shell echo echo $(VERSION_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 2))
	$(eval PATCH=$(shell echo echo $(VERSION_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 3))
	@echo "Version: $(VERSION_TAG)"
	@echo "Major: $(MAJOR)"
	@echo "Minor: $(MINOR)"
	@echo "Patch: $(PATCH)"
	@echo "DEPLOY_ENVIRONMENT: $(DEPLOY_ENVIRONMENT)"

	@# verify that variables are exported tag is set
	@[ -z "$(MAJOR)" ] && echo MAJOR is empty && exit 1 || echo "deploying $(MAJOR)"
	@[ -z "$(MINOR)" ] && echo MINOR is empty && exit 1 || echo "deploying $(MINOR)"
	@[ -z "$(PATCH)" ] && echo PATCH is empty && exit 1 || echo "deploying $(PATCH)"
	@[ -z "$(DEPLOY_ENVIRONMENT)" ] && echo DEPLOY_ENVIRONMENT is empty && exit 1 || echo "deploying $(DEPLOY_ENVIRONMENT)"

	@# build tags
	$(eval MAJOR_TAG=$(IMAGE_NAME):$(MAJOR))
	$(eval MINOR_TAG=$(IMAGE_NAME):$(MAJOR).$(MINOR).$(PATCH))
	$(eval LATEST_TAG=$(IMAGE_NAME):latest)
	
	@# Image name is passed as a parameter
	@echo $(IMAGE_NAME)
	@echo $(MAJOR_TAG)
	@echo $(MINOR_TAG)
	@echo $(LATEST_TAG)

	@# create docker image
	docker build \
		-t $(LATEST_TAG) -t $(MAJOR_TAG) -t $(MINOR_TAG) \
		.. \
		--build-arg BUILD_ENVIRONMENT=$(DEPLOY_ENVIRONMENT) \
		-f Dockerfile-integrated

	@# push the docker image to the repo
	@ echo pushing image to repo
	docker push $(IMAGE_NAME) --all-tags
