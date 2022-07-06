# check if psql is not installed.
if (which psql | length) == 0 {
    # TODO: make it prints to stderr, or use correct error make
    print "Error: psql is not installed."
    exit
}

if (which sqlx | length) == 0 {
    # TODO: make it prints to stderr, or use connect error make.
    print "Error: sqlx is not installed."
    print "Use:"
    print "     cargo install sqlx-cli --no-default-features --features native-tls,postgres"
    print "to install it."
    exit
}

let DB_USER = ($env | get -i "POSTGRES_USER" | default "postgres")
let DB_PASSWORD = ($env | get -i "POSTGRES_PASSWORD" | default "password")
let DB_NAME = ($env | get -i "POSTGRES_DB" | default "newsletter")
let DB_PORT = ($env | get -i "POSTGRES_PORT" | default 5432)

if (not ($env | get -i "SKIP_DOCKER" | default false | into bool)) {
    # Launch postgres using Docker
    docker run -e $'POSTGRES_USER=($DB_USER)' -e $'POSTGRES_PASSWORD=($DB_PASSWORD)' -e $'POSTGRES_DB=($DB_NAME)' -p $'($DB_PORT):5432' -d postgres postgres -N 1000
                                                                                                                                                            # ^ Increased maximum number of connections for testing purposes
}

# The following doesn't work, at least we should make it works in nushell
# docker run -e POSTGRES_USER=$'($DB_USER)' -e POSTGRES_PASSWORD=password -e POSTGRES_DB=postgres -p $'($DB_PORT):5432' -d postgres postgres -N 1000


# We need to wait for Postgres to be healthy before starting to run
# commands against it.
# So, keep pinging postgres until it's ready to accept commands
let-env PGPASSWORD = $DB_PASSWORD

# Try at most 10 times
# TODO: if all failed, raise out error.
1..10 | each while { |it|
    do -i {psql -h "localhost" -U $DB_USER -p $DB_PORT -d postgres -c '\q'}
    if $env.LAST_EXIT_CODE == 0 {
        $nothing
    } else {
        # TODO: make it prints to stderr.
        print "Postgres is still unavailable - sleeping"
        sleep 3sec
        $it
    }
}

print $"Postgres is up and running on ($DB_PORT) - running migrations now!"
let-env DATABASE_URL = $"postgres://($DB_USER):($DB_PASSWORD)@localhost:($DB_PORT)/($DB_NAME)"
sqlx database create
sqlx migrate run

print $"Postgres has been migrated, ready to go!"
