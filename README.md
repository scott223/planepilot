# planepilot
xplane11 logger and (future) autopilot

```mermaid
---
title: PlanePilot
---
classDiagram
    DataServer <|-- PlaneConnector : sends plane state
    Dashboard <|-- DataServer : dashboard retrieves all data
    PlanePilot <|-- PlanePilotUI : sends user inputs
    PlanePilot --|> PlanePilotUI : retrieves plane state
    PlaneConnector <|-- PlanePilot : sends setpoints
    PlaneConnector --|> PlanePilot : retrieves plane state
    DataServer <|-- PlanePilot : sends setpoints
    X-Plane11 <|-- PlaneConnector : sends setpoints
    X-Plane11 --|> PlaneConnector : retrieves plane state
    class DataServer{
        +Channels
        +Data
    }
    class Dashboard {

    }
    class PlaneConnector{

    }
    class PlanePilotUI {

    }
    class X-Plane11 {

    }
```

## Tech stack

### UI
* React
  * Redux Saga
  * Rechart

###  Dataserver
* Rust
  * Tokio async
  * Axum
  * Sqlx (sqlite)
