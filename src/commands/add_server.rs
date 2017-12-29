use ::ServerList;

command!(
    add_server(context, message, args) {
        let server_address = args.single::<String>()?;
        let alias = args.single::<String>().or_else(|_| {
            message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
        })?;

        let mut data = context.data.lock();
        let mut server_list = data.get_mut::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;

        match server_list.insert(alias.clone(), server_address) {
            None => {
                let _ = message.reply(&format!("successfully inserted with name {}", alias));
            },
            Some(old) => {
                let _ = message.reply(&format!("successfully overwrote {} from {}", alias, old));
            },
        }
        println!("inserted, current contents is {:?}", server_list);
    }
);
