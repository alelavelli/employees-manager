# Corporate Group

A Corporate Group is a set of Companies that share a group of Employees and Users.
It is defined by a `name` and the list of companies.

A Corporate Group can be created by any User, however, in the frontend app this option is available only in the admin panel.

## Create

`POST /api/corporate-group/`

Creates a new Corporate Group with the given name that must be unique for the entire application.

When the admin panel creates the corporate group, it is without companies and without assigned users.
Therefore, the next action the platform admin will do is to add at least a user to the corporate group defining its role.

## Delete

`DELETE /api/corporate-group/{id}`

Only corporate group admins can delete a corporate group.

This operation is destructive and any entity associated to it is deleted.
Hence, any company, user role, timesheet, expense request, project and so on will be deleted.

A preferred option is to deactivate the corporate group, in this way, any operation is blocked without destroying any entity.

## Update

`PATCH /api/corporate-group`

Update corporate group by changing its name and the companies associated with it.

Note that the current version replaces everything is provided.

## Activate / Deactivate

`PATCH /api/corporate-group/{id}/activate`
`PATCH /api/corporate-group/{id}/deactivate`

A Company Group can be deactivated instead of delete to preserve all the data.
All the operations on a deactivated corporate group will result into an AccessControlError mocking the non existence of the corporate group.

## Add User

`POST /api/corporate-group/{id}/user/{user_id}`

When a User is added in a corporate group a new document UserCorporateGroupRole is created indicating the role the user has in it.
Moreover, for each company in the corporate group a new document UserCompanyRole is created with the default role.
In this way, the user is able to see both the corporate group and the companies.

If specified in the request parameters, also the employment contract between the user and a single company in the corporate group is created.

## Remove User

`DELETE /api/corporate-group/{id}/user/{user_id}`

Removes the user from the corporate group deleting the roles documents and the employment contract.

## Update User

`PATCH /api/corporate-group/{id}/user/{user_id}`

Update the user in the corporate group by changing its role or its contract information.
