# Application fundamentals

Implementation of the elementary functionalities in the application like CRUD operations for Users, Company and Roles.
Each functionality must be compatible both via web app and sdk.

## User

The User is globally defined inside the application, it is created by the **Application Administrator** which is a special User that manages the entire application.
Therefore, username and email must be unique for the entire application.

A User can create or can be assigned to one or more Companies.
Inside a Company, the User has:

- Role: defines permissions to operations he can do inside the Company
- Job Title: job position inside the Company, however, it does not affect his permissions in the application

## Management team

A management team is a group of Users, independently from the Role they have in the Company.
The management team is responsible for approving requests for holydays and other employees requests.

## Company

Represents an actual company and has a set of Users assigned to it.
It is created by a User that becomes the owner and can perform any operation on it.

## User operations

### Application Administrator

- create new User: the application administrator creates a new User in the application creating a temporary password that needs to be reset at the first login.
- deactivate a User: the application administrator disable a User in the application. A deactivated User:
  - when tries to login a courtesy page is shown indicating that he has been deactivated
  - when tries to use api key an error message is returned indicating that he has been deactivated
  - any Company for which the User is owner is deactivated
  - any Service account associated to the Company is deactivated
- delete a User: the application administrator delete permanently a User in the application.
- nominate other application administrators

### User Company Standard User

- non administration operations in the application that depend on the available modules

### User Company Admin

- any operation of standard user
- define list of Job Titles.
  Note that if a Job Title has been removed from the list the Users inside the Company with it are updated and will have an empty field
- add existing Users in the Company
- remove existing Users from the Company.
  When a User is removed from the Company he will not be able to perform any operation in it.
  However, the past actions remains.
  For instance, its timesheets or holydays request are still visible from other Users in the Company.
- add and remove Users in the Management Team

### User Company Owner

- any operation of the Company Admin
- change the Role of any User in the Company
