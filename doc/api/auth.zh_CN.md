[English](auth.md)
# 验证相关API
## 获取服务器验证状态
* 路径: `/api/auth/status`、 `/auth/status`、 `/api/auth` 或 `/auth`
* 方法: `GET` 或 `POST`
* 鉴权: 无需
* 参数：无
* 返回结构[示例](auth/status.example.json)
* 返回结果的[JSON Schema](auth/status.zh_CN.json)
## 获取服务器公钥
* 路径: `/api/auth/pubkey`、 `/auth/pubkey`
* 方法: `GET` 或 `POST`
* 鉴权: 无需
## 新增用户
* 路径: `/api/auth/user/add`、 `/auth/user/add`
* 方法: `GET` 或 `POST`
* RESTful: `PUT /api/auth/user` 或 `PUT /auth/user`
* 鉴权: 一般需要（如服务器状态内的`has_root_user`为`false`则无需鉴权，如需要仅限管理员）
## 更新用户
* 路径: `/api/auth/user/update`、 `/auth/user/update`
* 方法: `GET` 或 `POST`
* RESTful: `PATCH /api/auth/user` 或 `PATCH /auth/user`
* 鉴权: 需要（仅管理员）
## 修改用户名字
* 路径: `/api/auth/user/change/name`、 `/auth/user/change/name`
* 方法: `GET` 或 `POST`
* 鉴权: 需要
## 修改用户密码
* 路径: `/api/auth/user/change/password`、 `/auth/user/change/password`
* 方法: `GET` 或 `POST`
* 鉴权: 需要
## 删除用户
* 路径: `/api/auth/user/delete`、 `/auth/user/delete`
* 方法: `GET` 或 `POST`
* RESTful: `DELETE /api/auth/user` 或 `DELETE /auth/user`
* 鉴权: 需要（仅管理员）
## 获取用户信息
* 路径: `/api/auth/user/info`、 `/auth/user/info`
* 方法: `GET` 或 `POST`
* RESTful: `GET /api/auth/user` 或 `GET /auth/user`
* 鉴权: 需要（其他用户信息仅管理员）
## 获取用户列表
* 路径: `/api/auth/user/list`、 `/auth/user/list`
* 方法: `GET` 或 `POST`
* 鉴权: 需要（仅管理员）
## 获取Token
* 路径: `/api/auth/token/add`、 `/auth/token/add`
* 方法: `GET` 或 `POST`
* RESTful: `PUT /api/auth/token` 或 `PUT /auth/token`
* 鉴权：不需要
## 移除Token
* 路径: `/api/auth/token/delete`、 `/auth/token/delete`
* 方法: `GET` 或 `POST`
* RESTful: `DELETE /api/auth/token` 或 `DELETE /auth/token`
* 鉴权：需要（删除其他用户的Token仅管理员）
