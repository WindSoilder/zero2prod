# if a redis container is running, print instructions to kill it and exit.
let running_container = ((docker ps --filter 'name=redis' --format `{{.ID}}`) | str trim)
if (not ($running_container | empty?)) {
    print "there is a redis container already running, kill it with \n"
    print $"           docker kill ($running_container)"
    exit
}

# Launch redis using docker
docker run -p "6379:6379" -d --name $"redis_(date format %s)" redis:7

print "Redis is ready to go!"
