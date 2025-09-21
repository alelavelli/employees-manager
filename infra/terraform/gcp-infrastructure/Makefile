SHELL=/bin/bash

BRANCH_NAME=$(shell git rev-parse --abbrev-ref HEAD)
DEPLOY_ENVIRONMENT=$(shell if [ $(findstring main, $(BRANCH_NAME)) ]; then \
	echo 'prod'; \
elif [ $(findstring pre, $(BRANCH_NAME)) ]; then \
	echo 'pre'; \
else \
	echo 'dev'; \
fi)

init:
	terraform init -backend-config=$(DEPLOY_ENVIRONMENT).state.config

plan:
	terraform plan -var-file=common.tfvars -var-file=$(DEPLOY_ENVIRONMENT).tfvars

apply:
	terraform apply -var-file=common.tfvars -var-file=$(DEPLOY_ENVIRONMENT).tfvars