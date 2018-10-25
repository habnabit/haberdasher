@0xaedec27f9ef61431;

struct Option(T) {
    union {
        none @0 :Void;
        some @1 :T;
    }
}

struct Map(Key, Value) {
    struct Entry {
        key @0 :Key;
        value @1 :Value;
    }

    entries @0 :List(Entry);
}

struct DateTime {
    seconds @0 :Int64;
    nanoseconds @1 :UInt32;
}

struct Individual {
    id @0 :Data;
    name @1 :Text;
}

struct Performer {
    union {
        individual @0 :Individual;
        service @1 :Void;
        myself @2 :Void;
    }
}

struct Message {
    performer @0 :Performer;
    at @1 :DateTime;
    union {
        text @2 :Text;
        pose @3 :Text;
    }
}

interface Venue {
    struct Kind {
        union {
            individual @0 :Individual;
            group :group {
                name @1 :Text;
            }
            service @2 :Void;
        }
    }
    struct Instance {
        name @0 :Text;
    }

    getId @0 () -> (id :Data);
    getKind @1 () -> (kind :Kind);
    getInstance @2 () -> (instance :Option(Instance));
    getLastMessage @3 (since :Option(DateTime)) -> (message :Message);
    subscribe @4 ();
    unsubscribe @5 ();
}

interface AgentServer {
    establish @0 (agent :Agent);
    publish @1 (venue :Venue, message :Message);
}

interface Agent {
    listVenues @0 (withUpdatesSince :Option(DateTime)) -> (venues :List(Venue));
    setSubscribeToNewVenues @1 (subscribe: Bool);
}
