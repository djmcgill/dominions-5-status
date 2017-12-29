use ::ServerList;

command!(
    list_servers(context, message, _args) {
        let data = context.data.lock();
        let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
        let _ = message.reply(&format!("{:?}", server_list));
    }
);
