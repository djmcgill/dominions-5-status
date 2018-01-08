CMD='docker run -it -rm --volume "/home/ec2-user/dominions-5-status/resources":"/usr/src/myapp/resources" dom-5-bot'

while true; do
    $CMD
    SLEEP 60
done
