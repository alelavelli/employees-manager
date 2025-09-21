SHELL=/bin/bash

BRANCH_NAME=$(shell git rev-parse --abbrev-ref HEAD)
DEPLOY_ENVIRONMENT=$(shell if [ $(findstring main, $(BRANCH_NAME)) ]; then \
			echo 'prod'; \
		elif [ $(findstring pre, $(BRANCH_NAME)) ]; then \
			echo 'pre'; \
		else \
		 	echo 'dev'; \
		fi)

setup:
	cp .hooks/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
	cp .hooks/pre-push .git/hooks/pre-push && chmod +x .git/hooks/pre-push

format:
	cargo fmt

lint:
	cargo clippy -- -Dwarnings

test:
	cargo test -- --test-threads=12

all: format lint test

doc: 
	cargo doc
	RUSTDOCFLAGS="--html-in-header katex-header.html" cargo doc --no-deps --open -p rlalgs -p rlenv

deploy-tag:
	# This rule reads the current version tag, creates a new one with
	# the increment according to the variable KIND

	@# check if KIND variable is set
	@[ -z "$(KIND)" ] && echo KIND is empty && exit 1 || echo "creating tag $(KIND)"

	@# check if KIND variable has the allowed value
	@if [ "$${KIND}" != "major" -a "$${KIND}" != "minor" -a "$${KIND}" != "patch" ]; then \
		echo "Error: KIND environment variable must be set to 'major', 'minor', 'patch' or 'beta'."; \
		exit 1; \
	fi

	@# we add a prefix to the tag to specify the deploy environment
	$(eval DEPLOY_ENVIRONMENT_SUFFIX = @$(DEPLOY_ENVIRONMENT))

	@# read the current tag and export the three kinds
	@# to retrieve the version levels, we separate them by white space
	@# to do that we need to replace . and -
	@# then we keep the words number 1, 2, and 3
	$(eval CURRENT_TAG=$(shell git describe --tags --abbrev=0 --match="*@$(DEPLOY_ENVIRONMENT)"))
	$(eval MAJOR=$(shell echo echo $(CURRENT_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 1))
	$(eval MINOR=$(shell echo echo $(CURRENT_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 2))
	$(eval PATCH=$(shell echo echo $(CURRENT_TAG) | cut -d '@' -f 1 | cut -d 'v' -f 2 | cut -d '.' -f 3))
	@echo "Version: $(CURRENT_TAG)"
	@echo "Major: $(MAJOR)"
	@echo "Minor: $(MINOR)"
	@echo "Patch: $(PATCH)"

	@# according to the kind set the new tag
	@# I know it's strange but if blocks must be written without indentation
ifeq ($(KIND),major)
	$(eval MAJOR := $(shell echo $$(($(MAJOR) + 1))))
	$(eval MINOR := 0)
	$(eval PATCH := 0)
else ifeq ($(KIND),minor)
	$(eval MINOR := $(shell echo $$(($(MINOR) + 1))))
	$(eval PATCH := 0)
else ifeq ($(KIND),patch)
	$(eval PATCH := $(shell echo $$(($(PATCH) + 1))))
endif

	@# Set the new tag variable
	$(eval NEW_TAG=v$(MAJOR).$(MINOR).$(PATCH)$(DEPLOY_ENVIRONMENT_SUFFIX))
	$(eval MESSAGE=new version $(NEW_TAG))

	@echo $(NEW_TAG)
	@# create new tag
	git tag -a $(NEW_TAG) -m "$(MESSAGE)"

	@# push the tag
	git push origin $(NEW_TAG)

deploy-tag-patch:
	@make deploy-tag KIND=patch

deploy-tag-minor:
	@make deploy-tag KIND=minor

deploy-tag-major:
	@make deploy-tag KIND=major
