[中文简体](auth.zh_CN.md)
# Authentication related APIs
## Get server authentication status
* route: `/auth/status` or `/api/auth/status`
* method: `GET` or `POST`
* auth: Not needed
* parameters: None
* [Example](auth/status.example.json) of the response
* [JSON schema](auth/status.json) of the response
## Get server public key
* route: `/auth/pubkey` or `/api/auth/pubkey`
* method: `GET` or `POST`
* auth: Not needed
* parameters: None
* [Example](auth/pubkey.example.json) of the response
* [JSON schema](auth/pubkey.json) of the response
## User management
### Add user
* route: `/auth/user/add` or `/api/auth/user/add`
* method: `GET` or `POST`
* RESTful: `PUT /api/auth/user` 或 `PUT /auth/user`
* auth: Needed (If `has_root_user` in server status is `true`, authentication is not needed. Admin privilege is needed)
* parameters:

| Name | Type | Required | Description |
|:---:|:---:|:---:|:---:|
| `name` | `string` | Yes | User's name |
| `username` | `string` | Yes | User's name (unique and used to login) |
| `password` | `string` | Yes | User's password (RSA encrypted) |
| `is_admin` | `boolean` | No | Whether the user is an admin Default: `false`. Root user will always have admin privillege. |
| `id` | `uint64` | No | The user's id which want to be modified |
* [Example](auth/user/add.example.json) of the response
* [JSON schema](auth/user/add.json) of the response
