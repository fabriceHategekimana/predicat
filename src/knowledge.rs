use sqlite::Connection;

//TODO: use it when the data test is no more needed
static _CREATE_FACTS : &str = " CREATE TABLE facts(
                  'subject' TEXT,
                  'link' TEXT,
                  'goal' TEXT,
                  PRIMARY KEY (subject,link,goal)
                ); ";

static _CREATE_RULES : &str = " CREATE TABLE rules_default(
                    'id' INTEGER,
                    'source' TEXT,
                    'type' TEXT, 
                    'listener' TEXT, 
                    'body' TEXT);
                    ";

static _CREATE_HISTORICAL : &str = " CREATE TABLE historical(
                            'stage' TEXT, 
                            'event' TEXT,
                            PRIMARY KEY (event)); 
                    ";


static _CREATE_MACRO : &str = " CREATE TABLE macro(
                            'name' TEXT,
                            'body' TEXT);
                    ";

static _CREATE_STAGE : &str = "
CREATE TABLE stage('stage' TEXT); 
";

static _CREATE_CONTEXT : &str = "
CREATE TABLE context('name' TEXT); 
";

static _CREATE_UNIQUE_INDEX_RULES : &str = "
CREATE UNIQUE INDEX rules_body on rules (body);
";

static _CREATE_UNIQUE_INDEX_FACTS : &str = "create unique index fact_subject_link_goal on rules (subject, link, goal);";

static _CREATE_UNIQUE_INDEX_HISTORICAL : &str = "
CREATE UNIQUE INDEX historical_event on historical (event);
";

static _INITIALYZE_STAGE : &str = "insert into stage (stage) values (0)";
static _INITIALYZE_CONTEXT : &str = "insert into context (name) values ('default')";

fn get(connection: Connection, query: &str) {
    let query = query.replace("from facts", "from facts_default");
    let _res = connection.iterate(query, |pairs| {
        for &(_, value) in pairs.iter() {
            println!("{}", value.unwrap());
        }
        true
    });
}

fn _modifier(connection: &Connection, query: &str) {
    connection.execute(query).unwrap();
}

pub fn initialisation(query: &str) {
    let connection = sqlite::open("data.db").unwrap();
    //modifier(&connection, CREATE_FACTS);
    get(connection, query);
}
