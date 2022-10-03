# 验证相关API
## 获取服务器验证状态
* 路径: `/api/auth/status`、 `/auth/status`、 `/api/auth` 或 `/auth`
* 方法: `GET` 或 `POST`
* 鉴权: 无需
## 获取服务器公钥
* 路径: `/api/auth/pubkey`、 `/auth/pubkey`
* 方法: `GET` 或 `POST`
* 鉴权: 无需
## 新增用户
* 路径: `/api/auth/user/add`、 `/auth/user/add`
* 方法: `GET` 或 `POST`
* RESTful: `PUT /api/auth/user` 或 `PUT /auth/user`
* 鉴权: 一般需要（如服务器状态内的`has_root_user`为`false`则无需鉴权）
## 更新用户
* 路径: `/api/auth/user/update`、 `/auth/user/update`
* 方法: `GET` 或 `POST`
* RESTful: `PATCH /api/auth/user` 或 `PATCH /auth/user`
* 鉴权: 需要
## 获取Token
* 路径: `/api/auth/token/add`、 `/auth/token/add`
* 方法: `GET` 或 `POST`
* RESTful: `PUT /api/auth/token` 或 `PUT /auth/token`
* 鉴权：不需要
