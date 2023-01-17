use sqlite::Connection;

static CREATE_FACTS : &str = " CREATE TABLE facts(
                  'subject' TEXT,
                  'link' TEXT,
                  'goal' TEXT,
                  PRIMARY KEY (subject,link,goal)
                ); ";

static CREATE_RULES : &str = " CREATE TABLE rules_default(
                    'id' INTEGER,
                    'source' TEXT,
                    'type' TEXT, 
                    'listener' TEXT, 
                    'body' TEXT);
                    ";

static CREATE_HISTORICAL : &str = " CREATE TABLE historical(
                            'stage' TEXT, 
                            'event' TEXT,
                            PRIMARY KEY (event)); 
                    ";


static CREATE_MACRO : &str = " CREATE TABLE macro(
                            'name' TEXT,
                            'body' TEXT);
                    ";

static CREATE_STAGE : &str = "
CREATE TABLE stage('stage' TEXT); 
";

static CREATE_CONTEXT : &str = "
CREATE TABLE context('name' TEXT); 
";

static CREATE_UNIQUE_INDEX_RULES : &str = "
CREATE UNIQUE INDEX rules_body on rules (body);
";

static CREATE_UNIQUE_INDEX_FACTS : &str = "create unique index fact_subject_link_goal on rules (subject, link, goal);";

static CREATE_UNIQUE_INDEX_HISTORICAL : &str = "
CREATE UNIQUE INDEX historical_event on historical (event);
";

static INITIALYZE_STAGE : &str = "insert into stage (stage) values (0)";
static INITIALYZE_CONTEXT : &str = "insert into context (name) values ('default')";

fn get(connection: Connection) {
    let query = "SELECT * FROM users WHERE age > 50";
    connection
        .iterate(query, |pairs| {
            for &(name, value) in pairs.iter() {
                println!("{} = {}", name, value.unwrap());
            }
            true
        })
        .unwrap();
}

fn modifier(connection: Connection, query: &str) {
    connection.execute(query).unwrap();
}

pub fn initialisation() {
    let connection = sqlite::open("data.db").unwrap();
    modifier(connection, CREATE_FACTS);
}
