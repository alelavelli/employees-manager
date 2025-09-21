import os
from dotenv import load_dotenv

from uuid import uuid4
from locust import HttpUser, task, between
from random import choice, choices, randint

load_dotenv()


class AdminUser(HttpUser):
    wait_time = between(1, 5)

    headers = {}

    http_user_id = ""

    user_counter = 0
    company_counter = 0
    project_counter = 0

    def on_start(self):
        """Retrieve login information from the environment, log and save JWT in header attribute"""
        username = os.environ["LOCUST_USERNAME"]
        password = os.environ["LOCUST_PASSWORD"]

        jwt_response = self.client.post(
            "/api/auth/login", json={"username": username, "password": password}
        ).json()
        jwt = jwt_response["token"]
        self.headers["Authorization"] = f"Bearer {jwt}"
        self.client.headers = self.headers

        self.http_user_id = str(uuid4())

    @task(10)
    def open_home(self):
        """Open the home page"""
        self.client.get("/api/auth/user")
        self.client.get("/api/company")
        self.client.get("/api/corporate-group")
        self.client.get("/api/notification")

    @task(3)
    def create_user(self):
        """Opens admin panel and create a new user"""
        # Step 1: open the admin panel
        self.client.get("/api/admin/overview")
        self.client.get("/api/admin/user")
        # Step 2: create new user
        self.client.post(
            "/api/admin/user",
            json={
                "email": f"{self.http_user_id}_user_{self.user_counter}@ml.com",
                "name": f"{self.http_user_id}_name_{self.user_counter}",
                "surname": f"{self.http_user_id}_surname_{self.user_counter}",
                "username": f"{self.http_user_id}_username_{self.user_counter}",
                "password": "1234Abch#!",
            },
        )
        self.user_counter += 1
        # Step 3: reload the page
        self.client.get("/api/admin/overview")
        self.client.get("/api/admin/user")

    @task(1)
    def create_company(self):
        """Opens home page and create a new company"""
        # Step 1: open home page
        self.client.get("/api/company")
        self.client.get("/api/corporate-group")
        # Step 2: create company
        self.client.post(
            "/api/company",
            json={
                "jobTitle": "CEO",
                "name": f"{self.http_user_id}_company_{self.company_counter}",
            },
        )
        self.company_counter += 1
        # Step 3: reload the page
        self.client.get("/api/company")
        self.client.get("/api/corporate-group")

    @task(3)
    def add_users_to_company(self):
        """Opens company page and add the user to the company
        by adding a random number of projects"""
        # Step 1: open company settings page
        companies = self.client.get("/api/company").json()
        if len(companies) > 0:
            # pick a random company
            company_id = choice(companies)["id"]
            self.client.get(f"/api/company/{company_id}/user", name="/company/id/user")
            self.client.get(
                f"/api/company/{company_id}/project", name="/company/id/project"
            )
            self.client.get(
                f"/api/company/{company_id}/pending-user",
                name="/company/id/pending-user",
            )
            self.client.get(
                f"/api/company/{company_id}/activity", name="/company/id/activity"
            )

            # Step 2: open invite user modal
            users_to_invite = self.client.get(
                f"/api/company/{company_id}/user-to-invite",
                name="/company/id/user-to-invite",
            ).json()
            projects = self.client.get(
                f"/api/company/{company_id}/project", name="/company/id/project"
            ).json()
            if len(users_to_invite) > 0:
                n_projects = len(projects)
                projects_to_add = [
                    project["id"]
                    for project in choices(projects, k=randint(0, n_projects))
                ]
                self.client.post(
                    f"/api/company/{company_id}/invite-user",
                    name="/company/id/invite-user",
                    json={
                        "jobTitle": "title",
                        "projectIds": projects_to_add,
                        "role": "User",
                        "userId": choice(users_to_invite)["userId"],
                    },
                )
                self.client.get(
                    f"/api/company/{company_id}/pending-user",
                    name="/company/id/pending-user",
                )

    @task(2)
    def create_project(self):
        """Open company page and create a new project"""
        # Step 1: open company settings page
        companies = self.client.get("/api/company").json()
        if len(companies) > 0:
            # pick a random company
            company_id = choice(companies)["id"]
            self.client.get(f"/api/company/{company_id}/user", name="/company/id/user")
            self.client.get(
                f"/api/company/{company_id}/project", name="/company/id/project"
            )
            self.client.get(
                f"/api/company/{company_id}/pending-user",
                name="/company/id/pending-user",
            )
            self.client.get(
                f"/api/company/{company_id}/activity", name="/company/id/activity"
            )

            # Step 2: create new project
            self.client.post(
                f"/api/company/{company_id}/project",
                name="/company/id/project",
                json={
                    "name": f"{self.http_user_id}_project_name_{self.project_counter}",
                    "code": f"{self.http_user_id}_project_code_{self.project_counter}",
                },
            )
