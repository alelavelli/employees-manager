# employees-manager

Repository that contains some functionalities to manage employees like recording working hour or asking for permissions holidays.

The structure is the following:

- a User registered into the portal can belong to one or more companies in which plays different roles, it could be the admin of a Company or a simple User
- a Company has a list of User that are part of it
- the Role identifies what operations a User can do for the Company but it differs from the job title the User has with it. For instance, an Employee can be Company Admin because he manages the portal but is not the CEO. Hence, the Role is considered only from the RBAC point of view
- Job Title represents the job the Employee has in the Company. A Company administrator defines the list of Job Title available for the Company
- when a registered User creates a Company it is the Owner and can add other existing User to the Company inviting them and assigning Admin or User role and a Job Title
- the Management team is a group of User inside the Company that are responsible to accept or deny requests from the Employees in the Company
- a Admin can add or remove User to Management team

## Holidays and permissions

- in this section of the portal, an Employee can ask for permissions or holidays to the Management team. Then, someone of the Management team will receive a notification and will accept or deny them.
- User in Management team can see summaries of the requests and download them a xlsx file
- User in Management team can assign holiday to all Employee User like Company holidays and they will be notified

## Timesheet

- Employees are assigned to one or more Project with a Job Title specific to the project
- Employee compile the timesheet by specifying for each hour, the Project he worked on and if it was at the office or in remote work
- Employee can also add expenses to be refunded and upload receipts
